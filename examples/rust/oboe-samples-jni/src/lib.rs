#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;

use oboe_android::aaudio::AAudioBackend;
use oboe_android::backend::AudioBackend;
use oboe_android::opensles::OpenSLESBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::{Error, Result};
use oboe_core::types::{AudioApi, Format, PerformanceMode};
use oboe_samples::drumthumper::DrumThumper;
use oboe_samples::hello_oboe::HelloOboeSample;
use oboe_samples::iolib::{OneShotSampleSource, SampleBuffer};
use oboe_samples::mega_drone::MegaDroneSynth;
use oboe_samples::minimal_oboe::SimpleNoiseMaker;
use oboe_samples::powerplay::PowerPlayPlayer;
use oboe_samples::shared::{mono_to_stereo, Oscillator};
use oboe_samples::sound_board::SoundBoardSynth;

#[allow(non_camel_case_types)]
type jint = i32;
#[allow(non_camel_case_types)]
type jobject = *mut c_void;
#[allow(non_camel_case_types)]
type jclass = *mut c_void;
#[allow(non_camel_case_types)]
type JNIEnv = *mut c_void;

const SAMPLE_RATE: i32 = 48_000;
const CHANNEL_COUNT: usize = 2;
const CHANNEL_COUNT_I32: i32 = 2;
const CHUNK_FRAMES: usize = 192;
const SAMPLE_COUNT: usize = 13;
const MIN_DURATION_MILLIS: i32 = 80;
const MAX_DURATION_MILLIS: i32 = 5_000;
const DEFAULT_TIMEOUT_NANOS: i64 = 100_000_000;

enum NativeSampleStream {
    AAudio(AAudioBackend),
    OpenSLES(OpenSLESBackend),
}

impl NativeSampleStream {
    fn open(api: AudioApi) -> Result<Self> {
        let builder = StreamBuilder {
            api,
            performance_mode: PerformanceMode::LowLatency,
            sample_rate: SAMPLE_RATE,
            channel_count: CHANNEL_COUNT_I32,
            format: Format::Float,
            ..StreamBuilder::default()
        };

        match api {
            AudioApi::AAudio | AudioApi::Unspecified => {
                AAudioBackend::open(&builder).map(Self::AAudio)
            }
            AudioApi::OpenSLES => OpenSLESBackend::open(&builder).map(Self::OpenSLES),
        }
    }

    fn request_start(&mut self) -> Result<()> {
        match self {
            Self::AAudio(stream) => stream.request_start(),
            Self::OpenSLES(stream) => stream.request_start(),
        }
    }

    fn request_stop(&mut self) -> Result<()> {
        match self {
            Self::AAudio(stream) => stream.request_stop(),
            Self::OpenSLES(stream) => stream.request_stop(),
        }
    }

    fn close(&mut self) -> Result<()> {
        match self {
            Self::AAudio(stream) => stream.close(),
            Self::OpenSLES(stream) => stream.close(),
        }
    }

    fn write_f32(&mut self, audio: &[f32]) -> Result<i32> {
        match self {
            Self::AAudio(stream) => stream.write_f32(audio, DEFAULT_TIMEOUT_NANOS),
            Self::OpenSLES(stream) => stream.write_f32(audio, DEFAULT_TIMEOUT_NANOS),
        }
    }
}

pub fn sample_count() -> usize {
    SAMPLE_COUNT
}

pub fn run_sample_for_test(sample_id: usize, audio_api: i32, duration_millis: i32) -> Result<i32> {
    run_sample(sample_id, api_from_i32(audio_api), duration_millis)
}

fn run_sample(sample_id: usize, api: AudioApi, duration_millis: i32) -> Result<i32> {
    if sample_id >= SAMPLE_COUNT {
        return Err(Error::InvalidArgument);
    }

    let duration_millis = duration_millis.clamp(MIN_DURATION_MILLIS, MAX_DURATION_MILLIS);
    let frame_count = (SAMPLE_RATE as usize * duration_millis as usize) / 1_000;
    let audio = render_sample(sample_id, frame_count)?;

    let mut stream = NativeSampleStream::open(api)?;
    if let Err(error) = stream.request_start() {
        let _ = stream.close();
        return Err(error);
    }

    let write_result = write_audio(&mut stream, api, &audio);
    let stop_result = stream.request_stop();
    let close_result = stream.close();

    let written = write_result?;
    stop_result?;
    close_result?;
    Ok(written)
}

fn write_audio(stream: &mut NativeSampleStream, api: AudioApi, audio: &[f32]) -> Result<i32> {
    if api == AudioApi::OpenSLES {
        let written = stream.write_f32(audio)?;
        return if written > 0 {
            Ok(written)
        } else {
            Err(Error::InvalidState)
        };
    }

    let mut written_total = 0_i32;
    for chunk in audio.chunks(CHUNK_FRAMES * CHANNEL_COUNT) {
        let written = stream.write_f32(chunk)?;
        if written <= 0 {
            return Err(Error::InvalidState);
        }
        written_total = written_total
            .checked_add(written)
            .ok_or(Error::InvalidState)?;
    }
    Ok(written_total)
}

fn render_sample(sample_id: usize, frame_count: usize) -> Result<Vec<f32>> {
    match sample_id {
        0 => render_hello_oboe(frame_count),
        1 => Ok(SimpleNoiseMaker::new(CHANNEL_COUNT, 0x0b0e).render(frame_count)),
        2 => Ok(render_live_effect(frame_count)),
        3 => render_mega_drone(frame_count),
        4 => render_sound_board(frame_count),
        5 => render_audio_device(frame_count),
        6 => render_drumthumper(frame_count),
        7 => render_powerplay(frame_count),
        8 => render_rhythm_game(frame_count),
        9 => render_iolib(frame_count),
        10 => render_parselib(frame_count),
        11 => render_shared(frame_count),
        12 => render_debug_utils(frame_count),
        _ => Err(Error::InvalidArgument),
    }
}

fn render_hello_oboe(frame_count: usize) -> Result<Vec<f32>> {
    let mut sample = HelloOboeSample::new(SAMPLE_RATE, CHANNEL_COUNT);
    sample.tap(true);
    Ok(scale(sample.render(frame_count), 0.12))
}

fn render_live_effect(frame_count: usize) -> Vec<f32> {
    let input = tone_stereo(frame_count, 330.0, 0.10);
    oboe_samples::live_effect::process_full_duplex(&input, CHANNEL_COUNT, frame_count)
}

fn render_mega_drone(frame_count: usize) -> Result<Vec<f32>> {
    let mut synth = MegaDroneSynth::new(SAMPLE_RATE, CHANNEL_COUNT);
    synth.tap(true);
    Ok(scale(synth.render(frame_count), 0.20))
}

fn render_sound_board(frame_count: usize) -> Result<Vec<f32>> {
    let mut synth = SoundBoardSynth::new(SAMPLE_RATE, CHANNEL_COUNT, 8);
    synth.note_on(3);
    Ok(scale(synth.render(frame_count), 0.5))
}

fn render_audio_device(frame_count: usize) -> Result<Vec<f32>> {
    let devices = [
        oboe_samples::audio_device::AudioDevice::new(1, "built-in mic", false, true),
        oboe_samples::audio_device::AudioDevice::new(2, "speaker", true, false),
    ];
    let output =
        oboe_samples::audio_device::select_first_output(&devices).ok_or(Error::InvalidState)?;
    Ok(tone_stereo(
        frame_count,
        220.0 + output.id as f32 * 20.0,
        0.08,
    ))
}

fn render_drumthumper(frame_count: usize) -> Result<Vec<f32>> {
    let mut drums = DrumThumper::new(CHANNEL_COUNT);
    drums.load_pad(0, decaying_hit(2_400), 1, -0.35)?;
    drums.load_pad(1, decaying_hit(1_200), 1, 0.35)?;
    drums.trigger(0)?;
    let mut output = drums.render(frame_count);
    if frame_count > 4_000 {
        drums.trigger(1)?;
        mix_in(
            &mut output,
            &drums.render(frame_count - 4_000),
            4_000 * CHANNEL_COUNT,
        );
    }
    Ok(output)
}

fn render_powerplay(frame_count: usize) -> Result<Vec<f32>> {
    let mut player = PowerPlayPlayer::new(CHANNEL_COUNT);
    player.load_track(0, tone_mono(frame_count.max(1), 392.0, 0.25), 1)?;
    player.start_playing(0, PerformanceMode::LowLatency)?;
    Ok(player.render(frame_count))
}

fn render_rhythm_game(frame_count: usize) -> Result<Vec<f32>> {
    let mut game = oboe_samples::rhythm_game::RhythmGame::new(vec![100, 200, 300], 35);
    let tap_gain = match game.tap(108) {
        oboe_samples::rhythm_game::TapResult::Good => 0.16,
        _ => 0.08,
    };
    Ok(tone_stereo(frame_count, 660.0, tap_gain))
}

fn render_iolib(frame_count: usize) -> Result<Vec<f32>> {
    let buffer = SampleBuffer::new(decaying_hit(frame_count.clamp(1, 4_800)), 1, 48_000)?;
    let mut source = OneShotSampleSource::new(buffer, 0.0);
    source.trigger();
    let mut output = vec![0.0; frame_count * CHANNEL_COUNT];
    source.mix_into(&mut output, CHANNEL_COUNT, frame_count);
    Ok(output)
}

fn render_parselib(frame_count: usize) -> Result<Vec<f32>> {
    let mono = tone_mono(frame_count.clamp(1, 4_800), 523.25, 0.18);
    let samples = mono
        .iter()
        .map(|sample| oboe_core::format::float_to_i16(*sample))
        .collect::<Vec<_>>();
    let wav = oboe_samples::parselib::write_test_wav_i16(1, 48_000, &samples);
    let parsed = oboe_samples::parselib::WavData::parse(&wav)?;
    Ok(mono_to_stereo(&parsed.frames))
}

fn render_shared(frame_count: usize) -> Result<Vec<f32>> {
    let mut oscillator = Oscillator::new(SAMPLE_RATE, 440.0, 0.12);
    oscillator.set_wave_on(true);
    let mut mono = vec![0.0; frame_count];
    oscillator.render_mono(&mut mono);
    Ok(mono_to_stereo(&mono))
}

fn render_debug_utils(frame_count: usize) -> Result<Vec<f32>> {
    let mut trace = oboe_samples::debug_utils::Trace::enabled();
    if !trace.begin_section("oboe-sample-render") {
        return Err(Error::InvalidState);
    }
    trace.end_section();
    Ok(tone_stereo(frame_count, 176.0, 0.06))
}

fn tone_stereo(frame_count: usize, frequency: f32, gain: f32) -> Vec<f32> {
    mono_to_stereo(&tone_mono(frame_count, frequency, gain))
}

fn tone_mono(frame_count: usize, frequency: f32, gain: f32) -> Vec<f32> {
    (0..frame_count)
        .map(|frame| {
            let phase = (frame as f32 * frequency * core::f32::consts::TAU) / SAMPLE_RATE as f32;
            phase.sin() * gain
        })
        .collect()
}

fn decaying_hit(frame_count: usize) -> Vec<f32> {
    (0..frame_count)
        .map(|frame| {
            let decay = 1.0 - (frame as f32 / frame_count.max(1) as f32);
            let phase = frame as f32 * 0.41;
            phase.sin() * decay * 0.35
        })
        .collect()
}

fn scale(mut audio: Vec<f32>, gain: f32) -> Vec<f32> {
    for sample in &mut audio {
        *sample *= gain;
    }
    audio
}

fn mix_in(output: &mut [f32], input: &[f32], start_sample: usize) {
    for (index, sample) in input.iter().enumerate() {
        let Some(slot) = output.get_mut(start_sample + index) else {
            break;
        };
        *slot += *sample;
    }
}

fn api_from_i32(audio_api: i32) -> AudioApi {
    match audio_api {
        1 => AudioApi::AAudio,
        2 => AudioApi::OpenSLES,
        _ => AudioApi::AAudio,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_samples_RustSampleRunner_nativeSampleCount(
    _env: JNIEnv,
    _class: jclass,
) -> jint {
    SAMPLE_COUNT as jint
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_samples_RustSampleRunner_nativeRunSample(
    _env: JNIEnv,
    _self: jobject,
    sample_id: jint,
    audio_api: jint,
    duration_millis: jint,
) -> jint {
    let Ok(sample_id) = usize::try_from(sample_id) else {
        return -1;
    };
    run_sample_for_test(sample_id, audio_api, duration_millis).unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sample_ids_render_frames_through_audio_backend() {
        for sample_id in 0..sample_count() {
            assert!(run_sample_for_test(sample_id, 1, 120).unwrap() > 0);
        }
    }

    #[test]
    fn invalid_sample_id_returns_error() {
        assert!(run_sample_for_test(sample_count(), 1, 120).is_err());
    }
}
