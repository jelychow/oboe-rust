#![deny(unsafe_op_in_unsafe_fn)]

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Once, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use oboe_android::aaudio::AAudioBackend;
use oboe_android::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::types::{AudioApi, Direction, Format, PerformanceMode, SharingMode};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::{AUTHORIZATION, CONTENT_TYPE};
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;

const DEFAULT_MODEL: &str = "gpt-realtime";
const DEFAULT_VOICE: &str = "marin";
const DEFAULT_INSTRUCTIONS: &str =
    "You are a concise realtime voice assistant. Reply in the user's language.";
const SAMPLE_RATE: i32 = 24_000;
const CHANNEL_COUNT: i32 = 1;
const FRAMES_PER_CHUNK: usize = 480;
const IO_TIMEOUT_NANOS: i64 = 100_000_000;
const MAX_TRANSCRIPT_CHARS: usize = 8_192;
const INPUT_LOG_INTERVAL_CHUNKS: u64 = 50;
const OUTPUT_LOG_INTERVAL_CHUNKS: u64 = 20;
const WAV_HEADER_BYTES: usize = 44;

type AppResult<T> = Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealtimeConfig {
    api_key: String,
    model: String,
    instructions: String,
}

impl RealtimeConfig {
    fn new(api_key: String, model: String, instructions: String) -> AppResult<Self> {
        let api_key = api_key.trim().to_owned();
        if api_key.is_empty() {
            return Err("OpenAI API key is required.".to_owned());
        }

        let model = if model.trim().is_empty() {
            DEFAULT_MODEL.to_owned()
        } else {
            model.trim().to_owned()
        };
        let instructions = if instructions.trim().is_empty() {
            DEFAULT_INSTRUCTIONS.to_owned()
        } else {
            instructions.trim().to_owned()
        };

        Ok(Self {
            api_key,
            model,
            instructions,
        })
    }
}

#[derive(Clone, Debug)]
struct StatusSnapshot {
    status: String,
    transcript: String,
    last_error: String,
    input_chunks_sent: u64,
    input_frames_sent: u64,
    output_chunks_played: u64,
    output_frames_played: u64,
    input_level: f32,
    output_level: f32,
}

impl Default for StatusSnapshot {
    fn default() -> Self {
        Self {
            status: "Stopped".to_owned(),
            transcript: String::new(),
            last_error: String::new(),
            input_chunks_sent: 0,
            input_frames_sent: 0,
            output_chunks_played: 0,
            output_frames_played: 0,
            input_level: 0.0,
            output_level: 0.0,
        }
    }
}

type SharedStatus = Arc<Mutex<StatusSnapshot>>;

struct RunningSession {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl RunningSession {
    fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RunningSession {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

#[derive(Default)]
struct RealtimeController {
    session: Option<RunningSession>,
    status: SharedStatus,
}

impl RealtimeController {
    fn start(&mut self, config: RealtimeConfig) -> jint {
        if self.session.is_some() {
            set_status(&self.status, "Already running");
            return 0;
        }

        android_log::info("start requested");
        reset_status(&self.status);
        set_status(&self.status, "Connecting");

        let status = self.status.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let handle = thread::spawn(move || {
            install_crypto_provider();
            let result = run_blocking_session(config, thread_stop, status.clone());
            if let Err(error) = result {
                android_log::error(&format!(
                    "session failed: {}",
                    redact_sensitive_text(&error)
                ));
                set_error(&status, &error);
            } else {
                android_log::info("session ended");
            }
        });

        self.session = Some(RunningSession {
            stop,
            handle: Some(handle),
        });
        0
    }

    fn stop(&mut self) -> jint {
        if let Some(session) = self.session.take() {
            android_log::info("stop requested");
            set_status(&self.status, "Stopping");
            session.stop();
        }
        set_status(&self.status, "Stopped");
        0
    }

    fn status(&self) -> String {
        lock_status(&self.status).status.clone()
    }

    fn transcript(&self) -> String {
        lock_status(&self.status).transcript.clone()
    }

    fn last_error(&self) -> String {
        lock_status(&self.status).last_error.clone()
    }
}

fn controller() -> &'static Mutex<RealtimeController> {
    static CONTROLLER: OnceLock<Mutex<RealtimeController>> = OnceLock::new();
    CONTROLLER.get_or_init(|| Mutex::new(RealtimeController::default()))
}

fn lock_controller() -> MutexGuard<'static, RealtimeController> {
    controller()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct RunningRecording {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<AppResult<()>>>,
}

#[derive(Clone, Debug, Default)]
struct NativeAudioStats {
    input_chunks: u64,
    input_frames: u64,
    output_chunks: u64,
    output_frames: u64,
    input_level: f32,
    output_level: f32,
}

type SharedNativeAudioStats = Arc<Mutex<NativeAudioStats>>;

struct NativeAudioController {
    playback_stop: Option<Arc<AtomicBool>>,
    recording: Option<RunningRecording>,
    stats: SharedNativeAudioStats,
    last_error: String,
}

impl Default for NativeAudioController {
    fn default() -> Self {
        Self {
            playback_stop: None,
            recording: None,
            stats: Arc::new(Mutex::new(NativeAudioStats::default())),
            last_error: String::new(),
        }
    }
}

impl NativeAudioController {
    fn play_pcm(&mut self, audio: Vec<u8>) -> jint {
        if audio.is_empty() {
            self.last_error = "TTS returned no PCM audio.".to_owned();
            return -1;
        }

        if let Some(stop) = self.playback_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }

        let stop = Arc::new(AtomicBool::new(false));
        self.playback_stop = Some(stop.clone());
        let stats = self.stats.clone();
        reset_native_output_level(&stats);
        self.last_error.clear();
        android_log::info(&format!(
            "native oboe TTS playback requested bytes={}",
            audio.len()
        ));
        thread::spawn(move || {
            let result = play_pcm16_with_oboe(&audio, stop.clone(), stats.clone());
            reset_native_output_level(&stats);
            let mut controller = lock_native_audio();
            if controller
                .playback_stop
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &stop))
            {
                controller.playback_stop = None;
            }
            if let Err(error) = result {
                android_log::error(&error);
                controller.last_error = error;
            }
        });
        0
    }

    fn stop_playback(&mut self) -> jint {
        if let Some(stop) = self.playback_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        0
    }

    fn start_recording(&mut self, path: String) -> jint {
        if self.recording.is_some() {
            self.last_error = "ASR recording is already running.".to_owned();
            return -1;
        }

        self.last_error.clear();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let stats = self.stats.clone();
        reset_native_input_level(&stats);
        android_log::info(&format!("native oboe ASR recording start path={path}"));
        let handle = thread::spawn(move || record_wav_with_oboe(path, thread_stop, stats));
        self.recording = Some(RunningRecording {
            stop,
            handle: Some(handle),
        });
        0
    }

    fn stop_recording(&mut self) -> jint {
        let Some(mut recording) = self.recording.take() else {
            return 0;
        };

        recording.stop.store(true, Ordering::Relaxed);
        let result = recording
            .handle
            .take()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err("ASR recording thread panicked.".to_owned()))
            })
            .unwrap_or(Ok(()));

        match result {
            Ok(()) => {
                android_log::info("native oboe ASR recording stopped");
                0
            }
            Err(error) => {
                android_log::error(&error);
                self.last_error = error;
                -2
            }
        }
    }

    fn last_error(&self) -> String {
        self.last_error.clone()
    }
}

fn native_audio() -> &'static Mutex<NativeAudioController> {
    static NATIVE_AUDIO: OnceLock<Mutex<NativeAudioController>> = OnceLock::new();
    NATIVE_AUDIO.get_or_init(|| Mutex::new(NativeAudioController::default()))
}

fn lock_native_audio() -> MutexGuard<'static, NativeAudioController> {
    native_audio()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_native_stats(stats: &SharedNativeAudioStats) -> MutexGuard<'_, NativeAudioStats> {
    stats
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_status(status: &SharedStatus) -> MutexGuard<'_, StatusSnapshot> {
    status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn reset_status(status: &SharedStatus) {
    let mut snapshot = lock_status(status);
    *snapshot = StatusSnapshot {
        status: "Connecting".to_owned(),
        transcript: String::new(),
        last_error: String::new(),
        input_chunks_sent: 0,
        input_frames_sent: 0,
        output_chunks_played: 0,
        output_frames_played: 0,
        input_level: 0.0,
        output_level: 0.0,
    };
}

fn set_status(status: &SharedStatus, next: &str) {
    let mut snapshot = lock_status(status);
    if snapshot.status != next {
        android_log::info(&format!("status={next}"));
    }
    snapshot.status = next.to_owned();
}

fn set_error(status: &SharedStatus, error: &str) {
    let error = redact_sensitive_text(error);
    android_log::error(&error);
    let mut snapshot = lock_status(status);
    snapshot.status = "Error".to_owned();
    snapshot.last_error = error;
}

fn append_transcript(status: &SharedStatus, delta: &str) {
    let mut snapshot = lock_status(status);
    snapshot.transcript.push_str(delta);
    if snapshot.transcript.len() > MAX_TRANSCRIPT_CHARS {
        let keep_from = snapshot.transcript.len() - MAX_TRANSCRIPT_CHARS;
        snapshot.transcript = snapshot.transcript[keep_from..].to_owned();
    }
}

fn newline_transcript(status: &SharedStatus) {
    let mut snapshot = lock_status(status);
    if !snapshot.transcript.ends_with('\n') {
        snapshot.transcript.push('\n');
    }
}

fn record_input_frames(status: &SharedStatus, frames: usize, level: f32) -> (u64, u64) {
    let mut snapshot = lock_status(status);
    snapshot.input_chunks_sent = snapshot.input_chunks_sent.saturating_add(1);
    snapshot.input_frames_sent = snapshot.input_frames_sent.saturating_add(frames as u64);
    snapshot.input_level = level;
    (snapshot.input_chunks_sent, snapshot.input_frames_sent)
}

fn record_output_frames(status: &SharedStatus, frames: usize, level: f32) -> (u64, u64) {
    let mut snapshot = lock_status(status);
    snapshot.output_chunks_played = snapshot.output_chunks_played.saturating_add(1);
    snapshot.output_frames_played = snapshot.output_frames_played.saturating_add(frames as u64);
    snapshot.output_level = level;
    (snapshot.output_chunks_played, snapshot.output_frames_played)
}

fn record_native_input_frames(
    stats: &SharedNativeAudioStats,
    frames: usize,
    level: f32,
) -> (u64, u64) {
    let mut snapshot = lock_native_stats(stats);
    snapshot.input_chunks = snapshot.input_chunks.saturating_add(1);
    snapshot.input_frames = snapshot.input_frames.saturating_add(frames as u64);
    snapshot.input_level = level;
    (snapshot.input_chunks, snapshot.input_frames)
}

fn record_native_output_frames(
    stats: &SharedNativeAudioStats,
    frames: usize,
    level: f32,
) -> (u64, u64) {
    let mut snapshot = lock_native_stats(stats);
    snapshot.output_chunks = snapshot.output_chunks.saturating_add(1);
    snapshot.output_frames = snapshot.output_frames.saturating_add(frames as u64);
    snapshot.output_level = level;
    (snapshot.output_chunks, snapshot.output_frames)
}

fn reset_native_input_level(stats: &SharedNativeAudioStats) {
    lock_native_stats(stats).input_level = 0.0;
}

fn reset_native_output_level(stats: &SharedNativeAudioStats) {
    lock_native_stats(stats).output_level = 0.0;
}

fn native_audio_stats_snapshot() -> NativeAudioStats {
    let stats = { lock_native_audio().stats.clone() };
    let snapshot = lock_native_stats(&stats).clone();
    snapshot
}

fn combined_stats() -> String {
    let realtime = {
        let controller = lock_controller();
        let snapshot = lock_status(&controller.status).clone();
        snapshot
    };
    let native = native_audio_stats_snapshot();
    format_stats(
        realtime
            .input_chunks_sent
            .saturating_add(native.input_chunks),
        realtime
            .input_frames_sent
            .saturating_add(native.input_frames),
        realtime
            .output_chunks_played
            .saturating_add(native.output_chunks),
        realtime
            .output_frames_played
            .saturating_add(native.output_frames),
        realtime.input_level.max(native.input_level),
        realtime.output_level.max(native.output_level),
    )
}

fn format_stats(
    input_chunks: u64,
    input_frames: u64,
    output_chunks: u64,
    output_frames: u64,
    input_level: f32,
    output_level: f32,
) -> String {
    format!(
        "Mic sent: {input_chunks} chunks / {input_frames} frames. Output played: {output_chunks} chunks / {output_frames} frames. Levels: mic {:.3}, output {:.3}.",
        input_level.clamp(0.0, 1.0),
        output_level.clamp(0.0, 1.0)
    )
}

fn install_crypto_provider() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn run_blocking_session(
    config: RealtimeConfig,
    stop: Arc<AtomicBool>,
    status: SharedStatus,
) -> AppResult<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .map_err(|error| format!("Failed to start Tokio runtime: {error}"))?;

    runtime.block_on(run_realtime_session(config, stop, status))
}

async fn run_realtime_session(
    config: RealtimeConfig,
    stop: Arc<AtomicBool>,
    status: SharedStatus,
) -> AppResult<()> {
    android_log::info(&format!(
        "connecting realtime websocket model={}",
        config.model
    ));
    let mut request = format!("wss://api.openai.com/v1/realtime?model={}", config.model)
        .into_client_request()
        .map_err(|error| format!("Failed to create WebSocket request: {error}"))?;

    let auth = HeaderValue::from_str(&format!("Bearer {}", config.api_key))
        .map_err(|error| format!("Invalid API key header: {error}"))?;
    request.headers_mut().insert(AUTHORIZATION, auth);
    request
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let (socket, _) = connect_async(request)
        .await
        .map_err(|error| format!("Realtime WebSocket connection failed: {error}"))?;
    android_log::info("realtime websocket connected");
    set_status(&status, "Connected");

    let (mut write, mut read) = socket.split();
    write
        .send(Message::Text(
            build_session_update(&config).to_string().into(),
        ))
        .await
        .map_err(|error| format!("Failed to send session.update: {error}"))?;
    android_log::info("session.update sent");

    let mut output = RealtimeAudioOutput::open()?;
    let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<Vec<f32>>();
    let mic = start_microphone_thread(audio_tx, stop.clone(), status.clone());

    loop {
        tokio::select! {
            maybe_audio = audio_rx.recv() => {
                match maybe_audio {
                    Some(audio) => {
                        let event = build_audio_append_event(&audio);
                        write
                            .send(Message::Text(event.to_string().into()))
                            .await
                            .map_err(|error| format!("Failed to stream microphone audio: {error}"))?;
                        let level = audio_level_f32(&audio);
                        let (chunks, frames) = record_input_frames(&status, audio.len(), level);
                        if chunks == 1 || chunks % INPUT_LOG_INTERVAL_CHUNKS == 0 {
                            android_log::info(&format!(
                                "microphone audio sent chunks={chunks} frames={frames}"
                            ));
                        }
                    }
                    None => break,
                }
            }
            maybe_message = read.next() => {
                match maybe_message {
                    Some(Ok(Message::Text(text))) => {
                        handle_server_event(text.as_ref(), &mut output, &status)?;
                    }
                    Some(Ok(Message::Binary(_))) => {}
                    Some(Ok(Message::Close(frame))) => {
                        android_log::info(&format!("realtime websocket closed by server: {frame:?}"));
                        break;
                    }
                    None => {
                        android_log::info("realtime websocket ended");
                        break;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        android_log::error(&format!("realtime websocket read failed: {error}"));
                        return Err(format!("Realtime WebSocket read failed: {error}"));
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
    }

    stop.store(true, Ordering::Relaxed);
    let _ = write.close().await;
    let _ = mic.join();
    output.close();
    Ok(())
}

pub fn build_session_update(config: &RealtimeConfig) -> Value {
    json!({
        "type": "session.update",
        "session": {
            "type": "realtime",
            "model": config.model,
            "instructions": config.instructions,
            "output_modalities": ["audio"],
            "audio": {
                "input": {
                    "format": {
                        "type": "audio/pcm",
                        "rate": SAMPLE_RATE
                    },
                    "turn_detection": {
                        "type": "semantic_vad"
                    }
                },
                "output": {
                    "format": {
                        "type": "audio/pcm",
                        "rate": SAMPLE_RATE
                    },
                    "voice": DEFAULT_VOICE
                }
            }
        }
    })
}

pub fn build_audio_append_event(audio: &[f32]) -> Value {
    let pcm = f32_to_pcm16_le(audio);
    json!({
        "type": "input_audio_buffer.append",
        "audio": STANDARD.encode(pcm)
    })
}

fn start_microphone_thread(
    audio_tx: mpsc::UnboundedSender<Vec<f32>>,
    stop: Arc<AtomicBool>,
    status: SharedStatus,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut input = match RealtimeAudioInput::open() {
            Ok(input) => input,
            Err(error) => {
                android_log::error(&error);
                set_error(&status, &error);
                return;
            }
        };

        let mut buffer = vec![0.0_f32; FRAMES_PER_CHUNK * CHANNEL_COUNT as usize];
        while !stop.load(Ordering::Relaxed) {
            match input.read(&mut buffer) {
                Ok(read) if read > 0 => {
                    if audio_tx.send(buffer[..read].to_vec()).is_err() {
                        break;
                    }
                }
                Ok(_) => thread::sleep(Duration::from_millis(5)),
                Err(error) => {
                    android_log::error(&error);
                    set_error(&status, &error);
                    break;
                }
            }
        }
        input.close();
    })
}

struct RealtimeAudioInput {
    stream: AAudioBackend,
}

impl RealtimeAudioInput {
    fn open() -> AppResult<Self> {
        let mut stream = AAudioBackend::open(&StreamBuilder {
            api: AudioApi::AAudio,
            direction: Direction::Input,
            sharing_mode: SharingMode::Shared,
            performance_mode: PerformanceMode::LowLatency,
            sample_rate: SAMPLE_RATE,
            channel_count: CHANNEL_COUNT,
            format: Format::Float,
            ..StreamBuilder::default()
        })
        .map_err(|error| format!("Failed to open AAudio input stream: {error:?}"))?;
        stream
            .request_start()
            .map_err(|error| format!("Failed to start AAudio input stream: {error:?}"))?;
        android_log::info(&format!(
            "AAudio input started sample_rate={SAMPLE_RATE} channels={CHANNEL_COUNT}"
        ));
        Ok(Self { stream })
    }

    fn read(&mut self, buffer: &mut [f32]) -> AppResult<usize> {
        self.stream
            .read_f32(buffer, IO_TIMEOUT_NANOS)
            .map(|read| read.max(0) as usize)
            .map_err(|error| format!("Failed to read microphone audio: {error:?}"))
    }

    fn close(&mut self) {
        let _ = self.stream.request_stop();
        let _ = self.stream.close();
    }
}

struct RealtimeAudioOutput {
    stream: AAudioBackend,
}

impl RealtimeAudioOutput {
    fn open() -> AppResult<Self> {
        let mut stream = AAudioBackend::open(&StreamBuilder {
            api: AudioApi::AAudio,
            direction: Direction::Output,
            sharing_mode: SharingMode::Shared,
            performance_mode: PerformanceMode::LowLatency,
            sample_rate: SAMPLE_RATE,
            channel_count: CHANNEL_COUNT,
            format: Format::Float,
            ..StreamBuilder::default()
        })
        .map_err(|error| format!("Failed to open AAudio output stream: {error:?}"))?;
        stream
            .request_start()
            .map_err(|error| format!("Failed to start AAudio output stream: {error:?}"))?;
        android_log::info(&format!(
            "AAudio output started sample_rate={SAMPLE_RATE} channels={CHANNEL_COUNT}"
        ));
        Ok(Self { stream })
    }

    fn write_pcm16_le(&mut self, audio: &[u8]) -> AppResult<()> {
        let samples = pcm16_le_to_f32(audio);
        if samples.is_empty() {
            return Ok(());
        }
        self.stream
            .write_f32(&samples, IO_TIMEOUT_NANOS)
            .map(|_| ())
            .map_err(|error| format!("Failed to write assistant audio: {error:?}"))
    }

    fn close(&mut self) {
        let _ = self.stream.request_stop();
        let _ = self.stream.close();
    }
}

fn play_pcm16_with_oboe(
    audio: &[u8],
    stop: Arc<AtomicBool>,
    stats: SharedNativeAudioStats,
) -> AppResult<()> {
    let mut output = RealtimeAudioOutput::open()?;
    for chunk in audio.chunks(FRAMES_PER_CHUNK * 2) {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        output.write_pcm16_le(chunk)?;
        let frames = chunk.len() / 2;
        record_native_output_frames(&stats, frames, audio_level_pcm16_le(chunk));
    }
    output.close();
    Ok(())
}

fn record_wav_with_oboe(
    path: String,
    stop: Arc<AtomicBool>,
    stats: SharedNativeAudioStats,
) -> AppResult<()> {
    let mut input = RealtimeAudioInput::open()?;
    let mut file = File::create(&path)
        .map_err(|error| format!("Failed to create ASR WAV file '{path}': {error}"))?;
    file.write_all(&[0_u8; WAV_HEADER_BYTES])
        .map_err(|error| format!("Failed to reserve ASR WAV header: {error}"))?;

    let mut data_bytes = 0_u32;
    let mut buffer = vec![0.0_f32; FRAMES_PER_CHUNK * CHANNEL_COUNT as usize];
    while !stop.load(Ordering::Relaxed) {
        match input.read(&mut buffer) {
            Ok(read) if read > 0 => {
                let pcm = f32_to_pcm16_le(&buffer[..read]);
                file.write_all(&pcm)
                    .map_err(|error| format!("Failed to write ASR WAV data: {error}"))?;
                data_bytes = data_bytes.saturating_add(pcm.len() as u32);
                record_native_input_frames(&stats, read, audio_level_f32(&buffer[..read]));
            }
            Ok(_) => thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                input.close();
                return Err(error);
            }
        }
    }

    write_wav_header(
        &mut file,
        data_bytes,
        SAMPLE_RATE as u32,
        CHANNEL_COUNT as u16,
    )?;
    input.close();
    if data_bytes == 0 {
        return Err("No microphone samples were captured.".to_owned());
    }
    Ok(())
}

fn write_wav_header(
    file: &mut File,
    data_bytes: u32,
    sample_rate: u32,
    channels: u16,
) -> AppResult<()> {
    let bits_per_sample = 16_u16;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * u32::from(block_align);
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("Failed to seek ASR WAV header: {error}"))?;
    file.write_all(b"RIFF")
        .map_err(|error| format!("Failed to write ASR WAV header: {error}"))?;
    file.write_all(&(36_u32.saturating_add(data_bytes)).to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV size: {error}"))?;
    file.write_all(b"WAVEfmt ")
        .map_err(|error| format!("Failed to write ASR WAV format: {error}"))?;
    file.write_all(&16_u32.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV fmt size: {error}"))?;
    file.write_all(&1_u16.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV audio format: {error}"))?;
    file.write_all(&channels.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV channels: {error}"))?;
    file.write_all(&sample_rate.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV sample rate: {error}"))?;
    file.write_all(&byte_rate.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV byte rate: {error}"))?;
    file.write_all(&block_align.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV block align: {error}"))?;
    file.write_all(&bits_per_sample.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV bits: {error}"))?;
    file.write_all(b"data")
        .map_err(|error| format!("Failed to write ASR WAV data marker: {error}"))?;
    file.write_all(&data_bytes.to_le_bytes())
        .map_err(|error| format!("Failed to write ASR WAV data size: {error}"))?;
    Ok(())
}

fn handle_server_event(
    text: &str,
    output: &mut RealtimeAudioOutput,
    status: &SharedStatus,
) -> AppResult<()> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| format!("Invalid Realtime event JSON: {error}"))?;
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    match event_type {
        "session.created" | "session.updated" => {
            log_server_event(event_type);
            set_status(status, "Connected");
        }
        "input_audio_buffer.speech_started" => {
            log_server_event(event_type);
            set_status(status, "Listening");
        }
        "input_audio_buffer.speech_stopped" | "input_audio_buffer.committed" => {
            log_server_event(event_type);
            set_status(status, "Thinking")
        }
        "response.created" | "response.output_item.added" | "response.output_item.created" => {
            log_server_event(event_type);
            set_status(status, "Responding")
        }
        "response.output_audio.delta" | "response.audio.delta" => {
            if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                let bytes = STANDARD
                    .decode(delta)
                    .map_err(|error| format!("Invalid assistant audio chunk: {error}"))?;
                let frames = bytes.len() / 2;
                let level = audio_level_pcm16_le(&bytes);
                output.write_pcm16_le(&bytes)?;
                let (chunks, total_frames) = record_output_frames(status, frames, level);
                if chunks == 1 || chunks % OUTPUT_LOG_INTERVAL_CHUNKS == 0 {
                    android_log::info(&format!(
                        "assistant audio played chunks={chunks} frames={total_frames}"
                    ));
                }
            }
        }
        "response.output_audio_transcript.delta"
        | "response.audio_transcript.delta"
        | "response.output_text.delta"
        | "response.text.delta" => {
            if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                append_transcript(status, delta);
            }
        }
        "response.output_audio_transcript.done" | "response.output_text.done" | "response.done" => {
            log_server_event(event_type);
            newline_transcript(status);
            set_status(status, "Connected");
        }
        "error" => {
            let message = value
                .pointer("/error/message")
                .or_else(|| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("Realtime API returned an error.");
            android_log::error(&format!("server error: {}", redact_sensitive_text(message)));
            set_error(status, message);
        }
        _ => {}
    }

    Ok(())
}

fn log_server_event(event_type: &str) {
    android_log::info(&format!("server event={event_type}"));
}

fn redact_sensitive_text(message: &str) -> String {
    let mut output = String::with_capacity(message.len());
    let mut rest = message;

    while let Some(index) = rest.find("sk-") {
        output.push_str(&rest[..index]);
        let candidate = &rest[index..];
        let token_end = candidate
            .find(|character: char| {
                character.is_whitespace()
                    || matches!(
                        character,
                        '"' | '\'' | '`' | ',' | '.' | ';' | ':' | ')' | '(' | ']' | '['
                    )
            })
            .unwrap_or(candidate.len());

        output.push_str("sk-***");
        rest = &candidate[token_end..];
    }

    output.push_str(rest);
    output
}

pub fn f32_to_pcm16_le(audio: &[f32]) -> Vec<u8> {
    let mut output = Vec::with_capacity(audio.len() * 2);
    for sample in audio {
        let clipped = sample.clamp(-1.0, 1.0);
        let scaled = if clipped < 0.0 {
            clipped * 32768.0
        } else {
            clipped * 32767.0
        };
        output.extend_from_slice(&(scaled.round() as i16).to_le_bytes());
    }
    output
}

pub fn pcm16_le_to_f32(audio: &[u8]) -> Vec<f32> {
    audio
        .chunks_exact(2)
        .map(|chunk| {
            let value = i16::from_le_bytes([chunk[0], chunk[1]]);
            if value < 0 {
                value as f32 / 32768.0
            } else {
                value as f32 / 32767.0
            }
        })
        .collect()
}

pub fn audio_level_f32(audio: &[f32]) -> f32 {
    if audio.is_empty() {
        return 0.0;
    }

    let sum_squares = audio
        .iter()
        .map(|sample| {
            let clipped = sample.clamp(-1.0, 1.0);
            clipped * clipped
        })
        .sum::<f32>();
    let rms = (sum_squares / audio.len() as f32).sqrt();
    (rms * 6.0).clamp(0.0, 1.0)
}

pub fn audio_level_pcm16_le(audio: &[u8]) -> f32 {
    let samples = pcm16_le_to_f32(audio);
    audio_level_f32(&samples)
}

fn jstring_to_string(env: &mut JNIEnv<'_>, value: JString<'_>) -> AppResult<String> {
    env.get_string(&value)
        .map(|value| value.into())
        .map_err(|error| format!("Failed to read Java string: {error}"))
}

fn string_to_jstring(env: &mut JNIEnv<'_>, value: String) -> jstring {
    env.new_string(value)
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[cfg(target_os = "android")]
mod android_log {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int};

    const ANDROID_LOG_INFO: c_int = 4;
    const ANDROID_LOG_ERROR: c_int = 6;
    const TAG: &str = "OpenAIRealtimeRust";

    #[link(name = "log")]
    extern "C" {
        fn __android_log_print(prio: c_int, tag: *const c_char, fmt: *const c_char, ...) -> c_int;
    }

    pub(super) fn info(message: &str) {
        print(ANDROID_LOG_INFO, message);
    }

    pub(super) fn error(message: &str) {
        print(ANDROID_LOG_ERROR, message);
    }

    fn print(priority: c_int, message: &str) {
        let Ok(tag) = CString::new(TAG) else {
            return;
        };
        let Ok(format) = CString::new("%s") else {
            return;
        };
        let Ok(message) = CString::new(message.replace('\0', " ")) else {
            return;
        };

        unsafe {
            __android_log_print(priority, tag.as_ptr(), format.as_ptr(), message.as_ptr());
        }
    }
}

#[cfg(not(target_os = "android"))]
mod android_log {
    pub(super) fn info(_message: &str) {}

    pub(super) fn error(_message: &str) {}
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_startNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    api_key: JString<'_>,
    model: JString<'_>,
    instructions: JString<'_>,
) -> jint {
    let api_key = match jstring_to_string(&mut env, api_key) {
        Ok(value) => value,
        Err(error) => {
            set_error(&lock_controller().status, &error);
            return -1;
        }
    };
    let model = jstring_to_string(&mut env, model).unwrap_or_default();
    let instructions = jstring_to_string(&mut env, instructions).unwrap_or_default();

    let config = match RealtimeConfig::new(api_key, model, instructions) {
        Ok(config) => config,
        Err(error) => {
            set_error(&lock_controller().status, &error);
            return -2;
        }
    };

    lock_controller().start(config)
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_stopNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jint {
    lock_controller().stop()
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_playPcmNative(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    pcm: JByteArray<'_>,
) -> jint {
    let audio = match env.convert_byte_array(pcm) {
        Ok(audio) => audio,
        Err(error) => {
            lock_native_audio().last_error = format!("Failed to read TTS PCM byte array: {error}");
            return -1;
        }
    };
    lock_native_audio().play_pcm(audio)
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_stopNativeAudio(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jint {
    let mut audio = lock_native_audio();
    let playback = audio.stop_playback();
    let recording = audio.stop_recording();
    if playback != 0 {
        playback
    } else {
        recording
    }
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_startWavRecordingNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    path: JString<'_>,
) -> jint {
    let path = match jstring_to_string(&mut env, path) {
        Ok(path) => path,
        Err(error) => {
            lock_native_audio().last_error = error;
            return -1;
        }
    };
    lock_native_audio().start_recording(path)
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_stopWavRecordingNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jint {
    lock_native_audio().stop_recording()
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_nativeAudioErrorNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let error = lock_native_audio().last_error();
    string_to_jstring(&mut env, redact_sensitive_text(&error))
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_statusNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let status = lock_controller().status();
    string_to_jstring(&mut env, status)
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_transcriptNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let transcript = lock_controller().transcript();
    string_to_jstring(&mut env, transcript)
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_lastErrorNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let error = lock_controller().last_error();
    string_to_jstring(&mut env, error)
}

#[no_mangle]
pub extern "system" fn Java_com_example_openairustrealtime_RealtimeNative_statsNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let stats = combined_stats();
    string_to_jstring(&mut env, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_openai_key() -> String {
        ["sk", "test-fixture"].join("-")
    }

    #[test]
    fn config_uses_defaults_without_persisting_key() {
        let config =
            RealtimeConfig::new(format!(" {} ", fake_openai_key()), " ".into(), " ".into())
                .unwrap();
        assert_eq!(config.model, DEFAULT_MODEL);
        assert_eq!(config.instructions, DEFAULT_INSTRUCTIONS);
        assert_eq!(config.api_key, fake_openai_key());
    }

    #[test]
    fn session_update_uses_ga_realtime_shape() {
        let config = RealtimeConfig::new(
            fake_openai_key(),
            "gpt-realtime".into(),
            "short replies".into(),
        )
        .unwrap();
        let event = build_session_update(&config);
        assert_eq!(event["type"], "session.update");
        assert_eq!(event["session"]["type"], "realtime");
        assert_eq!(event["session"]["model"], "gpt-realtime");
        assert_eq!(event["session"]["audio"]["input"]["format"]["rate"], 24_000);
        assert_eq!(
            event["session"]["audio"]["output"]["format"]["rate"],
            24_000
        );
        assert_eq!(event["session"]["audio"]["output"]["voice"], DEFAULT_VOICE);
    }

    #[test]
    fn audio_append_event_encodes_pcm16_payload() {
        let event = build_audio_append_event(&[-1.0, 0.0, 1.0]);
        let payload = event["audio"].as_str().unwrap();
        let bytes = STANDARD.decode(payload).unwrap();
        assert_eq!(bytes, vec![0x00, 0x80, 0x00, 0x00, 0xff, 0x7f]);
    }

    #[test]
    fn pcm16_conversion_round_trips_basic_values() {
        let bytes = f32_to_pcm16_le(&[-1.0, -0.5, 0.0, 0.5, 1.0]);
        let samples = pcm16_le_to_f32(&bytes);
        assert!((samples[0] + 1.0).abs() < 0.0001);
        assert!((samples[1] + 0.5).abs() < 0.0001);
        assert_eq!(samples[2], 0.0);
        assert!((samples[3] - 0.5).abs() < 0.0001);
        assert!((samples[4] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn audio_level_tracks_sample_energy() {
        assert_eq!(audio_level_f32(&[0.0, 0.0, 0.0]), 0.0);
        let quiet = audio_level_f32(&[0.02, -0.02, 0.02, -0.02]);
        let loud = audio_level_f32(&[0.40, -0.40, 0.40, -0.40]);
        assert!(quiet > 0.0, "quiet non-silent audio should be visible");
        assert!(loud > quiet, "louder audio should produce a higher level");
    }

    #[test]
    fn transcript_status_appends_and_limits_text() {
        let status = SharedStatus::default();
        append_transcript(&status, "hello");
        newline_transcript(&status);
        append_transcript(&status, "world");

        let snapshot = lock_status(&status);
        assert_eq!(snapshot.transcript, "hello\nworld");
    }

    #[test]
    fn redacts_openai_keys_from_errors() {
        let fake_key = ["sk", "test123"].join("-");
        let fake_project_key = ["sk", "proj", "abc"].join("-");
        let redacted = redact_sensitive_text(&format!(
            "Incorrect API key provided: {fake_key}. Check {fake_project_key}"
        ));
        assert_eq!(redacted, "Incorrect API key provided: sk-***. Check sk-***");
    }
}
