#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use oboe_android::aaudio::AAudioBackend;
use oboe_android::backend::AudioBackend;
#[cfg(test)]
use oboe_android::fake::FakeBackend;
use oboe_android::opensles::OpenSLESBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::types::{AudioApi, Format, PerformanceMode, SharingMode};
use oboe_samples::minimal_oboe::SimpleNoiseMaker;

#[allow(non_camel_case_types)]
type jint = i32;
#[allow(non_camel_case_types)]
type jobject = *mut c_void;
#[allow(non_camel_case_types)]
type JNIEnv = *mut c_void;

const CHANNEL_COUNT: usize = 2;
const CHANNEL_COUNT_I32: i32 = 2;
const SAMPLE_RATE: i32 = 48_000;
const FRAMES_PER_CHUNK: usize = 192;
const WRITE_TIMEOUT_NANOS: i64 = 100_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerState {
    Started,
    Stopped,
}

enum BackendStream {
    AAudio(AAudioBackend),
    OpenSLES(OpenSLESBackend),
    #[cfg(test)]
    Fake(FakeBackend),
}

impl BackendStream {
    fn open(audio_api: AudioApi) -> oboe_core::error::Result<Self> {
        let builder = StreamBuilder {
            api: audio_api,
            sharing_mode: SharingMode::Exclusive,
            performance_mode: PerformanceMode::LowLatency,
            sample_rate: SAMPLE_RATE,
            channel_count: CHANNEL_COUNT_I32,
            format: Format::Float,
            ..StreamBuilder::default()
        };

        match audio_api {
            AudioApi::AAudio | AudioApi::Unspecified => {
                AAudioBackend::open(&builder).map(Self::AAudio)
            }
            AudioApi::OpenSLES => OpenSLESBackend::open(&builder).map(Self::OpenSLES),
        }
    }

    #[cfg(test)]
    fn open_for_test() -> oboe_core::error::Result<Self> {
        FakeBackend::open(&StreamBuilder::default()).map(Self::Fake)
    }

    fn request_start(&mut self) -> oboe_core::error::Result<()> {
        match self {
            Self::AAudio(stream) => stream.request_start(),
            Self::OpenSLES(stream) => stream.request_start(),
            #[cfg(test)]
            Self::Fake(stream) => stream.request_start(),
        }
    }

    fn request_stop(&mut self) -> oboe_core::error::Result<()> {
        match self {
            Self::AAudio(stream) => stream.request_stop(),
            Self::OpenSLES(stream) => stream.request_stop(),
            #[cfg(test)]
            Self::Fake(stream) => stream.request_stop(),
        }
    }

    fn close(&mut self) -> oboe_core::error::Result<()> {
        match self {
            Self::AAudio(stream) => stream.close(),
            Self::OpenSLES(stream) => stream.close(),
            #[cfg(test)]
            Self::Fake(stream) => stream.close(),
        }
    }

    fn write_f32(&mut self, audio: &[f32]) -> oboe_core::error::Result<i32> {
        match self {
            Self::AAudio(stream) => stream.write_f32(audio, WRITE_TIMEOUT_NANOS),
            Self::OpenSLES(stream) => stream.write_f32(audio, WRITE_TIMEOUT_NANOS),
            #[cfg(test)]
            Self::Fake(stream) => stream.write_f32(audio, WRITE_TIMEOUT_NANOS),
        }
    }
}

pub struct SimpleNoisePlayer {
    state: PlayerState,
    worker: Option<NoiseWorker>,
    audio_api: AudioApi,
    test_mode: bool,
}

impl SimpleNoisePlayer {
    fn new(audio_api: AudioApi) -> Self {
        Self {
            state: PlayerState::Stopped,
            worker: None,
            audio_api,
            test_mode: false,
        }
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        Self {
            state: PlayerState::Stopped,
            worker: None,
            audio_api: AudioApi::AAudio,
            test_mode: true,
        }
    }

    pub fn state(&self) -> PlayerState {
        self.state
    }

    pub fn start(&mut self) -> jint {
        if self.state == PlayerState::Started {
            return 0;
        }

        match self.open_worker() {
            Ok(worker) => {
                self.worker = Some(worker);
                self.state = PlayerState::Started;
                0
            }
            Err(_) => -1,
        }
    }

    pub fn stop(&mut self) -> jint {
        if let Some(worker) = self.worker.take() {
            worker.stop();
        }
        self.state = PlayerState::Stopped;
        0
    }

    #[cfg(test)]
    fn on_error_after_close_for_test(&mut self) -> jint {
        let was_started = self.state == PlayerState::Started;
        self.stop();
        if was_started {
            self.start()
        } else {
            0
        }
    }

    fn open_worker(&self) -> oboe_core::error::Result<NoiseWorker> {
        let mut stream = if self.test_mode {
            #[cfg(test)]
            {
                BackendStream::open_for_test()?
            }
            #[cfg(not(test))]
            {
                BackendStream::open(self.audio_api)?
            }
        } else {
            BackendStream::open(self.audio_api)?
        };

        stream.request_start()?;
        Ok(NoiseWorker::start(stream))
    }
}

struct NoiseWorker {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl NoiseWorker {
    fn start(mut stream: BackendStream) -> Self {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop = stop.clone();
        let handle = thread::spawn(move || {
            let mut generator = SimpleNoiseMaker::new(CHANNEL_COUNT, 0x5eed);
            while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
                let audio = generator.render(FRAMES_PER_CHUNK);
                if stream.write_f32(&audio).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(4));
            }
            let _ = stream.request_stop();
            let _ = stream.close();
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }

    fn stop(mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for NoiseWorker {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn player() -> &'static Mutex<SimpleNoisePlayer> {
    static PLAYER: OnceLock<Mutex<SimpleNoisePlayer>> = OnceLock::new();
    PLAYER.get_or_init(|| Mutex::new(SimpleNoisePlayer::new(AudioApi::AAudio)))
}

fn lock_player() -> MutexGuard<'static, SimpleNoisePlayer> {
    player()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[no_mangle]
pub extern "system" fn Java_com_example_minimaloboe_AudioPlayer_startAudioStreamNative(
    _env: JNIEnv,
    _self: jobject,
) -> jint {
    lock_player().start()
}

#[no_mangle]
pub extern "system" fn Java_com_example_minimaloboe_AudioPlayer_stopAudioStreamNative(
    _env: JNIEnv,
    _self: jobject,
) -> jint {
    lock_player().stop()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_start_stop_matches_minimaloboe_lifecycle() {
        let mut player = SimpleNoisePlayer::new_for_test();
        assert_eq!(player.state(), PlayerState::Stopped);

        assert_eq!(player.start(), 0);
        assert_eq!(player.state(), PlayerState::Started);
        assert_eq!(player.start(), 0);
        assert_eq!(player.state(), PlayerState::Started);

        assert_eq!(player.stop(), 0);
        assert_eq!(player.state(), PlayerState::Stopped);
        assert_eq!(player.stop(), 0);
        assert_eq!(player.state(), PlayerState::Stopped);
    }

    #[test]
    fn player_error_after_close_restarts_when_previously_started() {
        let mut player = SimpleNoisePlayer::new_for_test();
        assert_eq!(player.start(), 0);
        assert_eq!(player.on_error_after_close_for_test(), 0);
        assert_eq!(player.state(), PlayerState::Started);
    }
}
