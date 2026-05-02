#![cfg_attr(not(test), no_std)]

mod aaudio;
mod extensions;
mod fifo;
mod opensles;
mod resampler;
mod stream;

use core::ptr;

const AUDIO_FORMAT_I16: i32 = 1;
const AUDIO_FORMAT_FLOAT: i32 = 2;
const AUDIO_FORMAT_I24: i32 = 3;
const AUDIO_FORMAT_I32: i32 = 4;
const AUDIO_FORMAT_IEC61937: i32 = 5;
const AUDIO_FORMAT_MP3: i32 = 6;
const AUDIO_FORMAT_AAC_LC: i32 = 7;
const AUDIO_FORMAT_AAC_HE_V1: i32 = 8;
const AUDIO_FORMAT_AAC_HE_V2: i32 = 9;
const AUDIO_FORMAT_AAC_ELD: i32 = 10;
const AUDIO_FORMAT_AAC_XHE: i32 = 11;
const AUDIO_FORMAT_OPUS: i32 = 12;

const SCALE_I16_TO_FLOAT: f32 = 1.0 / 32768.0;
const SCALE_I24_TO_FLOAT: f32 = 1.0 / 2147483648.0;
const SCALE_FLOAT_TO_I16: f32 = 32768.0;
const SCALE_FLOAT_TO_I24: f32 = 8388608.0;
const SCALE_FLOAT_TO_I32: f32 = 2147483648.0;
const I24_PACKED_MAX: i32 = 0x007F_FFFF;
const I24_PACKED_MIN: i32 = -0x0080_0000;
const SQRT_2: f32 = core::f32::consts::SQRT_2;

const LIMITER_POLYNOMIAL_A: f32 = -0.603_553_4;
const LIMITER_POLYNOMIAL_B: f32 = 2.207_106_8;
const LIMITER_POLYNOMIAL_C: f32 = -0.603_553_4;
const LIMITER_X_WHEN_Y_IS_3_DECIBELS: f32 = 1.828_427_1;

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn oboe_rust_convert_format_to_size_in_bytes(format: i32) -> i32 {
    match format {
        AUDIO_FORMAT_I16 | AUDIO_FORMAT_IEC61937 => 2,
        AUDIO_FORMAT_FLOAT | AUDIO_FORMAT_I32 => 4,
        AUDIO_FORMAT_I24 => 3,
        AUDIO_FORMAT_MP3
        | AUDIO_FORMAT_AAC_LC
        | AUDIO_FORMAT_AAC_HE_V1
        | AUDIO_FORMAT_AAC_HE_V2
        | AUDIO_FORMAT_AAC_ELD
        | AUDIO_FORMAT_AAC_XHE
        | AUDIO_FORMAT_OPUS => 0,
        _ => 0,
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_convert_float_to_pcm16(
    source: *const f32,
    destination: *mut i16,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    for i in 0..num_samples as usize {
        let mut sample = ((*source.add(i) + 1.0) * SCALE_FLOAT_TO_I16) as i32;
        sample = sample.clamp(0, 0x0FFFF);
        *destination.add(i) = (sample - 32768) as i16;
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_convert_pcm16_to_float(
    source: *const i16,
    destination: *mut f32,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    oboe_rust_source_i16_to_float(source, destination, num_samples);
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_source_i16_to_float(
    source: *const i16,
    destination: *mut f32,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    for i in 0..num_samples as usize {
        *destination.add(i) = *source.add(i) as f32 * SCALE_I16_TO_FLOAT;
    }
}

#[no_mangle]
/// # Safety
///
/// `source` must be valid for `num_samples * 3` contiguous bytes and `destination` must be
/// valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_source_i24_to_float(
    source: *const u8,
    destination: *mut f32,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    for i in 0..num_samples as usize {
        let offset = i * 3;
        let sample_bits = ((*source.add(offset + 2) as u32) << 24)
            | ((*source.add(offset + 1) as u32) << 16)
            | ((*source.add(offset) as u32) << 8);
        *destination.add(i) = (sample_bits as i32) as f32 * SCALE_I24_TO_FLOAT;
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_sink_float_to_i16(
    source: *const f32,
    destination: *mut i16,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    for i in 0..num_samples as usize {
        *destination.add(i) = float_to_i16_sample(*source.add(i));
    }
}

#[no_mangle]
/// # Safety
///
/// `source` must be valid for `num_samples` contiguous samples and `destination` must be
/// valid for `num_samples * 3` contiguous bytes.
pub unsafe extern "C" fn oboe_rust_sink_float_to_i24(
    source: *const f32,
    destination: *mut u8,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    for i in 0..num_samples as usize {
        let sample = float_to_i24_sample(*source.add(i)) as u32;
        let offset = i * 3;
        *destination.add(offset) = sample as u8;
        *destination.add(offset + 1) = (sample >> 8) as u8;
        *destination.add(offset + 2) = (sample >> 16) as u8;
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_sink_float_to_i32(
    source: *const f32,
    destination: *mut i32,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    for i in 0..num_samples as usize {
        *destination.add(i) = float_to_i32_sample(*source.add(i));
    }
}

#[no_mangle]
/// # Safety
///
/// `source` must be valid for `num_frames` contiguous samples and `destination` must be valid
/// for `num_frames * channel_count` contiguous samples.
pub unsafe extern "C" fn oboe_rust_mono_to_multi(
    source: *const f32,
    destination: *mut f32,
    num_frames: i32,
    channel_count: i32,
) {
    if source.is_null() || destination.is_null() || num_frames <= 0 || channel_count <= 0 {
        return;
    }
    let channel_count = channel_count as usize;
    for frame in 0..num_frames as usize {
        let sample = *source.add(frame);
        let frame_offset = frame * channel_count;
        for channel in 0..channel_count {
            *destination.add(frame_offset + channel) = sample;
        }
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_copy_float_buffer(
    source: *const f32,
    destination: *mut f32,
    num_samples: i32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    ptr::copy_nonoverlapping(source, destination, num_samples as usize);
}

#[no_mangle]
/// # Safety
///
/// `source` must be valid for `num_frames * input_channel_count` contiguous samples and
/// `destination` must be valid for `num_frames` contiguous samples.
pub unsafe extern "C" fn oboe_rust_multi_to_mono(
    source: *const f32,
    destination: *mut f32,
    num_frames: i32,
    input_channel_count: i32,
) {
    if source.is_null() || destination.is_null() || num_frames <= 0 || input_channel_count <= 0 {
        return;
    }
    let input_channel_count = input_channel_count as usize;
    for frame in 0..num_frames as usize {
        *destination.add(frame) = *source.add(frame * input_channel_count);
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_frames * channel_count` contiguous samples.
pub unsafe extern "C" fn oboe_rust_mono_blend(
    source: *const f32,
    destination: *mut f32,
    num_frames: i32,
    channel_count: i32,
    inv_channel_count: f32,
) {
    if source.is_null() || destination.is_null() || num_frames <= 0 || channel_count <= 0 {
        return;
    }
    let channel_count = channel_count as usize;
    for frame in 0..num_frames as usize {
        let frame_offset = frame * channel_count;
        let mut accum = 0.0;
        for channel in 0..channel_count {
            accum += *source.add(frame_offset + channel);
        }
        let blended = accum * inv_channel_count;
        for channel in 0..channel_count {
            *destination.add(frame_offset + channel) = blended;
        }
    }
}

#[no_mangle]
/// # Safety
///
/// `source` must be valid for `num_frames` samples and `destination` must be valid for
/// `num_frames * channel_count` samples.
pub unsafe extern "C" fn oboe_rust_many_to_multi_channel(
    source: *const f32,
    destination: *mut f32,
    num_frames: i32,
    channel_count: i32,
    channel: i32,
) {
    if source.is_null()
        || destination.is_null()
        || num_frames <= 0
        || channel_count <= 0
        || channel < 0
        || channel >= channel_count
    {
        return;
    }
    let channel_count = channel_count as usize;
    let channel = channel as usize;
    for frame in 0..num_frames as usize {
        *destination.add(frame * channel_count + channel) = *source.add(frame);
    }
}

#[no_mangle]
/// # Safety
///
/// `source` must be valid for `num_frames * channel_count` samples and `destination` must be
/// valid for `num_frames` samples.
pub unsafe extern "C" fn oboe_rust_multi_to_many_channel(
    source: *const f32,
    destination: *mut f32,
    num_frames: i32,
    channel_count: i32,
    channel: i32,
) {
    if source.is_null()
        || destination.is_null()
        || num_frames <= 0
        || channel_count <= 0
        || channel < 0
        || channel >= channel_count
    {
        return;
    }
    let channel_count = channel_count as usize;
    let channel = channel as usize;
    for frame in 0..num_frames as usize {
        *destination.add(frame) = *source.add(frame * channel_count + channel);
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_frames * channel_count` contiguous samples.
/// `remaining_frames` must be a valid mutable pointer to the active ramp frame counter.
pub unsafe extern "C" fn oboe_rust_ramp_linear(
    source: *const f32,
    destination: *mut f32,
    num_frames: i32,
    channel_count: i32,
    level_to: f32,
    remaining_frames: *mut i32,
    scaler: f32,
) {
    if source.is_null()
        || destination.is_null()
        || remaining_frames.is_null()
        || num_frames <= 0
        || channel_count <= 0
    {
        return;
    }

    let channel_count = channel_count as usize;
    let mut source_index = 0usize;
    let mut destination_index = 0usize;
    let mut frames_left = num_frames;
    let mut remaining = *remaining_frames;

    if remaining > 0 {
        let frames_to_ramp = frames_left.min(remaining);
        frames_left -= frames_to_ramp;
        for _ in 0..frames_to_ramp {
            let current_level = level_to - (remaining as f32 * scaler);
            for _ in 0..channel_count {
                *destination.add(destination_index) = *source.add(source_index) * current_level;
                source_index += 1;
                destination_index += 1;
            }
            remaining -= 1;
        }
        *remaining_frames = remaining;
    }

    let samples_left = frames_left as usize * channel_count;
    for _ in 0..samples_left {
        *destination.add(destination_index) = *source.add(source_index) * level_to;
        source_index += 1;
        destination_index += 1;
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_clip_to_range(
    source: *const f32,
    destination: *mut f32,
    num_samples: i32,
    minimum: f32,
    maximum: f32,
) {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return;
    }
    for i in 0..num_samples as usize {
        *destination.add(i) = clip_to_range(*source.add(i), minimum, maximum);
    }
}

#[no_mangle]
/// # Safety
///
/// `source` and `destination` must be valid for `num_samples` contiguous samples.
pub unsafe extern "C" fn oboe_rust_limiter_process_buffer(
    source: *const f32,
    destination: *mut f32,
    num_samples: i32,
    last_valid_output: f32,
) -> f32 {
    if source.is_null() || destination.is_null() || num_samples <= 0 {
        return last_valid_output;
    }

    let mut current = last_valid_output;
    for i in 0..num_samples as usize {
        let input = *source.add(i);
        if !input.is_nan() {
            current = limiter_process_float(input);
        }
        *destination.add(i) = current;
    }
    current
}

fn float_to_i16_sample(sample: f32) -> i16 {
    let scaled = (sample * SCALE_FLOAT_TO_I16) as i32;
    scaled.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn float_to_i24_sample(sample: f32) -> i32 {
    let scaled = (sample * SCALE_FLOAT_TO_I24) as i32;
    scaled.clamp(I24_PACKED_MIN, I24_PACKED_MAX)
}

fn float_to_i32_sample(sample: f32) -> i32 {
    if sample <= -1.0 {
        return i32::MIN;
    }
    if sample >= 1.0 {
        return i32::MAX;
    }
    let scaled = sample * SCALE_FLOAT_TO_I32;
    if sample > 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    }
}

fn clip_to_range(sample: f32, minimum: f32, maximum: f32) -> f32 {
    let at_least_minimum = if minimum < sample { sample } else { minimum };
    if maximum < at_least_minimum {
        maximum
    } else {
        at_least_minimum
    }
}

fn limiter_process_float(input: f32) -> f32 {
    let input_abs = input.abs();
    if input_abs <= 1.0 {
        return input;
    }

    let mut output = if input_abs < LIMITER_X_WHEN_Y_IS_3_DECIBELS {
        (LIMITER_POLYNOMIAL_A * input_abs + LIMITER_POLYNOMIAL_B) * input_abs + LIMITER_POLYNOMIAL_C
    } else {
        SQRT_2
    };
    if input < 0.0 {
        output = -output;
    }
    output
}

#[cfg(test)]
mod tests {
    use crate::aaudio::{
        oboe_rust_aaudio_adjust_input_capacity, oboe_rust_aaudio_calculate_latency_millis,
        oboe_rust_aaudio_coerce_open_result, oboe_rust_aaudio_force_starting_to_started,
        oboe_rust_aaudio_normalize_input_preset, oboe_rust_aaudio_request_already_satisfied,
        oboe_rust_aaudio_session_performance_mode, oboe_rust_aaudio_spatialization_behavior,
    };
    use crate::extensions::{
        oboe_rust_aaudio_callback_return_result,
        oboe_rust_aaudio_callback_should_launch_stop_thread, oboe_rust_mmap_enabled_from_policy,
        oboe_rust_mmap_load_symbols_result, oboe_rust_mmap_policy_enabled,
        oboe_rust_mmap_unavailable_result,
    };
    use crate::opensles::{
        oboe_rust_opensles_channel_mask_default, oboe_rust_opensles_configured_callback_frames,
        oboe_rust_opensles_convert_input_preset, oboe_rust_opensles_convert_oboe_performance_mode,
        oboe_rust_opensles_convert_opensl_performance_mode,
        oboe_rust_opensles_convert_output_usage,
        oboe_rust_opensles_estimate_native_frames_per_burst, oboe_rust_opensles_input_channel_mask,
        oboe_rust_opensles_normalize_input_preset, oboe_rust_opensles_optimal_buffer_queue_length,
        oboe_rust_opensles_output_channel_mask, oboe_rust_opensles_output_position_millis,
        oboe_rust_opensles_select_default_format,
    };
    use crate::resampler::{
        oboe_rust_integer_ratio_reduce, oboe_rust_linear_resampler_read_frame,
        oboe_rust_polyphase_resampler_read_frame, oboe_rust_sinc_resampler_read_frame,
    };
    use crate::stream::{
        oboe_rust_builder_is_compatible, oboe_rust_builder_select_backend,
        oboe_rust_builder_will_use_aaudio, oboe_rust_data_callback_should_continue,
        oboe_rust_stream_available_frames, oboe_rust_stream_default_delay_before_close_millis,
        oboe_rust_stream_optimal_buffer_size, oboe_rust_stream_wait_transition_result,
    };

    use super::*;

    #[test]
    fn maps_audio_formats_to_sample_sizes() {
        assert_eq!(
            oboe_rust_convert_format_to_size_in_bytes(AUDIO_FORMAT_I16),
            2
        );
        assert_eq!(
            oboe_rust_convert_format_to_size_in_bytes(AUDIO_FORMAT_FLOAT),
            4
        );
        assert_eq!(
            oboe_rust_convert_format_to_size_in_bytes(AUDIO_FORMAT_I24),
            3
        );
        assert_eq!(
            oboe_rust_convert_format_to_size_in_bytes(AUDIO_FORMAT_I32),
            4
        );
        assert_eq!(
            oboe_rust_convert_format_to_size_in_bytes(AUDIO_FORMAT_IEC61937),
            2
        );
        assert_eq!(
            oboe_rust_convert_format_to_size_in_bytes(AUDIO_FORMAT_MP3),
            0
        );
        assert_eq!(oboe_rust_convert_format_to_size_in_bytes(-1), 0);
    }

    #[test]
    fn converts_float_to_i16_like_flowgraph_sink() {
        let input = [1.0, 0.5, -0.25, -1.0, 0.0, 53.9, -87.2];
        let mut output = [0i16; 7];
        unsafe {
            oboe_rust_sink_float_to_i16(input.as_ptr(), output.as_mut_ptr(), input.len() as i32);
        }
        assert_eq!(output, [32767, 16384, -8192, -32768, 0, 32767, -32768]);
    }

    #[test]
    fn round_trips_packed_i24_samples() {
        let input = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x5A];
        let mut floats = [0.0f32; 3];
        let mut output = [0u8; 9];
        unsafe {
            oboe_rust_source_i24_to_float(input.as_ptr(), floats.as_mut_ptr(), 3);
            oboe_rust_sink_float_to_i24(floats.as_ptr(), output.as_mut_ptr(), 3);
        }
        assert_eq!(output, input);
    }

    #[test]
    fn clips_to_range_like_cpp_min_max_sequence() {
        let input = [-9.7, 0.5, -0.25, 1.0, 12.3, f32::NAN];
        let mut output = [0.0f32; 6];
        unsafe {
            oboe_rust_clip_to_range(input.as_ptr(), output.as_mut_ptr(), 6, -2.0, 1.5);
        }
        assert_eq!(&output[0..5], &[-2.0, 0.5, -0.25, 1.0, 1.5]);
        assert_eq!(output[5], -2.0);
    }

    #[test]
    fn limiter_reuses_last_valid_output_for_nan() {
        let input = [f32::NAN, 0.5, f32::NAN, f32::NAN, -10.0, f32::NAN];
        let mut output = [0.0f32; 6];
        let last = unsafe {
            oboe_rust_limiter_process_buffer(input.as_ptr(), output.as_mut_ptr(), 6, 0.0)
        };
        assert_eq!(last, -SQRT_2);
        assert_eq!(output, [0.0, 0.5, 0.5, 0.5, -SQRT_2, -SQRT_2]);
    }

    #[test]
    fn copies_and_routes_float_channels() {
        let input = [1.0, 2.0, 3.0, 4.0];
        let mut copy = [0.0; 4];
        unsafe {
            oboe_rust_copy_float_buffer(input.as_ptr(), copy.as_mut_ptr(), 4);
        }
        assert_eq!(copy, input);

        let stereo = [1.0, 10.0, 2.0, 20.0, 3.0, 30.0];
        let mut mono = [0.0; 3];
        unsafe {
            oboe_rust_multi_to_mono(stereo.as_ptr(), mono.as_mut_ptr(), 3, 2);
        }
        assert_eq!(mono, [1.0, 2.0, 3.0]);

        let mut right = [0.0; 3];
        unsafe {
            oboe_rust_multi_to_many_channel(stereo.as_ptr(), right.as_mut_ptr(), 3, 2, 1);
        }
        assert_eq!(right, [10.0, 20.0, 30.0]);

        let mut interleaved = [0.0; 6];
        unsafe {
            oboe_rust_many_to_multi_channel(mono.as_ptr(), interleaved.as_mut_ptr(), 3, 2, 0);
            oboe_rust_many_to_multi_channel(right.as_ptr(), interleaved.as_mut_ptr(), 3, 2, 1);
        }
        assert_eq!(interleaved, stereo);
    }

    #[test]
    fn blends_multichannel_frames_to_mono_per_frame() {
        let input = [1.0, 3.0, -2.0, 2.0, 6.0, 10.0];
        let mut output = [0.0; 6];
        unsafe {
            oboe_rust_mono_blend(input.as_ptr(), output.as_mut_ptr(), 2, 3, 1.0 / 3.0);
        }
        assert_eq!(output, [2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 6.0, 6.0, 6.0]);
    }

    #[test]
    fn applies_linear_ramp_and_then_target_level() {
        let input = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0];
        let mut output = [0.0; 6];
        let mut remaining = 2;
        unsafe {
            oboe_rust_ramp_linear(
                input.as_ptr(),
                output.as_mut_ptr(),
                3,
                2,
                1.0,
                &mut remaining,
                0.25,
            );
        }
        assert_eq!(remaining, 0);
        assert_eq!(output, [0.5, 1.0, 3.0, 6.0, 16.0, 32.0]);
    }

    #[test]
    fn reduces_resampler_rate_ratio_in_rust() {
        let mut numerator = 48_000;
        let mut denominator = 44_100;
        unsafe {
            oboe_rust_integer_ratio_reduce(&mut numerator, &mut denominator);
        }
        assert_eq!((numerator, denominator), (160, 147));
    }

    #[test]
    fn interpolates_linear_resampler_frame_in_rust() {
        let previous = [0.0, 10.0];
        let current = [10.0, 20.0];
        let mut output = [0.0; 2];
        unsafe {
            oboe_rust_linear_resampler_read_frame(
                previous.as_ptr(),
                current.as_ptr(),
                output.as_mut_ptr(),
                2,
                1,
                4,
            );
        }
        assert_eq!(output, [2.5, 12.5]);
    }

    #[test]
    fn reads_polyphase_and_sinc_frames_in_rust() {
        let x = [1.0, 10.0, 2.0, 20.0, 3.0, 30.0, 4.0, 40.0];
        let coefficients = [0.25, 0.75];
        let mut output = [0.0; 2];
        unsafe {
            oboe_rust_polyphase_resampler_read_frame(
                x.as_ptr(),
                coefficients.as_ptr(),
                output.as_mut_ptr(),
                2,
                2,
                0,
            );
        }
        assert_eq!(output, [1.75, 17.5]);

        let coefficients_high = [0.5, 0.5];
        unsafe {
            oboe_rust_sinc_resampler_read_frame(
                x.as_ptr(),
                coefficients.as_ptr(),
                coefficients_high.as_ptr(),
                output.as_mut_ptr(),
                2,
                2,
                0.25,
            );
        }
        assert_eq!(output, [1.6875, 16.875]);
    }

    #[test]
    fn applies_stream_builder_and_state_policy_in_rust() {
        assert!(oboe_rust_builder_will_use_aaudio(2, true, false));
        assert!(oboe_rust_builder_will_use_aaudio(0, true, true));
        assert_eq!(oboe_rust_builder_select_backend(0, 0, true, true), 1);
        assert_eq!(oboe_rust_builder_select_backend(1, 1, true, true), 3);
        assert!(oboe_rust_builder_is_compatible(
            0, 2, 0, 2, 48_000, 2, 192, 2
        ));
        assert!(!oboe_rust_builder_is_compatible(
            44_100, 2, 0, 2, 48_000, 2, 192, 2
        ));

        assert_eq!(
            oboe_rust_stream_wait_transition_result(12, 3, 4, 0, 4),
            -869
        );
        assert_eq!(oboe_rust_stream_wait_transition_result(3, 3, 4, 0, 4), 0);
        assert_eq!(
            oboe_rust_stream_wait_transition_result(3, 3, 4, -885, 3),
            -885
        );
        assert_eq!(oboe_rust_stream_wait_transition_result(2, 3, 4, 0, 2), -895);

        let mut available = 0;
        let result = unsafe { oboe_rust_stream_available_frames(10, 42, &mut available) };
        assert_eq!(result, 0);
        assert_eq!(available, 32);

        assert_eq!(
            oboe_rust_stream_default_delay_before_close_millis(192, 48_000, 10, 500),
            10
        );
        assert_eq!(
            oboe_rust_stream_optimal_buffer_size(1, 12, 4096, 192, 2),
            4096
        );
        assert_eq!(
            oboe_rust_stream_optimal_buffer_size(0, 12, 4096, 192, 2),
            384
        );
        assert!(oboe_rust_data_callback_should_continue(0));
        assert!(!oboe_rust_data_callback_should_continue(1));
    }

    #[test]
    fn applies_aaudio_callback_and_mmap_extension_policy_in_rust() {
        assert_eq!(oboe_rust_aaudio_callback_return_result(0, true, 30, 30), 0);
        assert!(oboe_rust_aaudio_callback_should_launch_stop_thread(
            1, true, 30, 30
        ));
        assert_eq!(oboe_rust_aaudio_callback_return_result(1, true, 30, 30), 0);
        assert!(!oboe_rust_aaudio_callback_should_launch_stop_thread(
            1, true, 31, 30
        ));
        assert_eq!(oboe_rust_aaudio_callback_return_result(1, true, 31, 30), 1);
        assert_eq!(
            oboe_rust_aaudio_callback_return_result(99, false, 30, 30),
            1
        );

        assert!(!oboe_rust_mmap_policy_enabled(1));
        assert!(oboe_rust_mmap_policy_enabled(2));
        assert!(oboe_rust_mmap_policy_enabled(3));
        assert!(oboe_rust_mmap_enabled_from_policy(0, true));
        assert!(!oboe_rust_mmap_enabled_from_policy(0, false));
        assert!(oboe_rust_mmap_enabled_from_policy(2, false));
        assert_eq!(oboe_rust_mmap_unavailable_result(), -889);
        assert_eq!(
            oboe_rust_mmap_load_symbols_result(true, true, true, true, true),
            0
        );
        assert_eq!(
            oboe_rust_mmap_load_symbols_result(true, false, true, true, true),
            -889
        );
    }

    #[test]
    fn applies_aaudio_backend_policy_in_rust() {
        assert_eq!(
            oboe_rust_aaudio_adjust_input_capacity(128, 1, 1, 12, 12, 0, 4096, true),
            4096
        );
        assert_eq!(
            oboe_rust_aaudio_adjust_input_capacity(8192, 1, 1, 12, 12, 0, 4096, true),
            8192
        );
        assert_eq!(
            oboe_rust_aaudio_session_performance_mode(12, 7, -1, 0, 0, 12, 10, true),
            10
        );
        assert_eq!(
            oboe_rust_aaudio_session_performance_mode(12, -1, -1, 0, 0, 12, 10, true),
            12
        );
        assert_eq!(
            oboe_rust_aaudio_normalize_input_preset(10, 28, 28, 10, 6),
            6
        );
        assert_eq!(
            oboe_rust_aaudio_normalize_input_preset(10, 29, 28, 10, 6),
            10
        );
        assert_eq!(oboe_rust_aaudio_spatialization_behavior(0, 0, 2, true), 2);
        assert_eq!(oboe_rust_aaudio_spatialization_behavior(1, 0, 2, true), 1);
        assert_eq!(oboe_rust_aaudio_spatialization_behavior(1, 0, 2, false), 2);
        assert_eq!(oboe_rust_aaudio_coerce_open_result(5, true, -896), -896);
        assert_eq!(oboe_rust_aaudio_coerce_open_result(5, false, -896), 5);
        assert_eq!(oboe_rust_aaudio_force_starting_to_started(true, 3, 3, 4), 4);
        assert!(oboe_rust_aaudio_request_already_satisfied(27, 27, 3, 3, 4));
        assert!(!oboe_rust_aaudio_request_already_satisfied(28, 27, 3, 3, 4));
        assert_eq!(
            oboe_rust_aaudio_calculate_latency_millis(
                true,
                48_480,
                48_000,
                2_000_000_000,
                2_000_000_000,
                48_000,
                1_000_000_000,
                1_000_000
            ),
            10.0
        );
        assert_eq!(
            oboe_rust_aaudio_calculate_latency_millis(
                false,
                48_480,
                48_000,
                2_000_000_000,
                2_000_000_000,
                48_000,
                1_000_000_000,
                1_000_000
            ),
            -10.0
        );
    }

    #[test]
    fn applies_opensles_backend_policy_in_rust() {
        assert_eq!(
            oboe_rust_opensles_channel_mask_default(2, 23, 24, 30, 0, 0x8000_0000u32 as i32),
            0b11
        );
        assert_eq!(
            oboe_rust_opensles_channel_mask_default(2, 24, 24, 30, 0, 0x8000_0000u32 as i32),
            0x8000_0003u32 as i32
        );
        assert_eq!(
            oboe_rust_opensles_channel_mask_default(31, 24, 24, 30, 0, 0x8000_0000u32 as i32),
            0
        );
        assert_eq!(
            oboe_rust_opensles_input_channel_mask(1, 99, 0x4, 0x1, 0x2),
            0x1
        );
        assert_eq!(
            oboe_rust_opensles_input_channel_mask(2, 99, 0x4, 0x1, 0x2),
            0x3
        );
        assert_eq!(
            oboe_rust_opensles_output_channel_mask(6, 99, 0x4, 0x3, 0x33, 0x3F, 0x63F),
            0x3F
        );

        assert_eq!(
            oboe_rust_opensles_optimal_buffer_queue_length(2, 8, 0, 2, 192, 192),
            2
        );
        assert_eq!(
            oboe_rust_opensles_optimal_buffer_queue_length(2, 8, 1500, 2, 192, 192),
            8
        );
        assert_eq!(
            oboe_rust_opensles_estimate_native_frames_per_burst(
                192, 48_000, 0, 10, 27, 25, 12, 20, 1000
            ),
            960
        );
        assert_eq!(
            oboe_rust_opensles_estimate_native_frames_per_burst(
                0, 44_100, 22_050, 12, 27, 25, 12, 20, 1000
            ),
            16
        );
        assert_eq!(oboe_rust_opensles_configured_callback_frames(0, 240), 240);
        assert_eq!(
            oboe_rust_opensles_output_position_millis(48_000, 48_000, 1000),
            1000
        );

        assert_eq!(
            oboe_rust_opensles_select_default_format(
                0,
                22,
                23,
                AUDIO_FORMAT_I16,
                AUDIO_FORMAT_FLOAT
            ),
            AUDIO_FORMAT_I16
        );
        assert_eq!(
            oboe_rust_opensles_select_default_format(
                0,
                23,
                23,
                AUDIO_FORMAT_I16,
                AUDIO_FORMAT_FLOAT
            ),
            AUDIO_FORMAT_FLOAT
        );
        assert_eq!(oboe_rust_opensles_normalize_input_preset(10, 10, 6), 6);
        assert_eq!(
            oboe_rust_opensles_convert_input_preset(7, 0, 1, 5, 6, 7, 9),
            7
        );
        assert_eq!(
            oboe_rust_opensles_convert_output_usage(14, 3, 0, 1, 2, 4, 5),
            3
        );
        assert_eq!(
            oboe_rust_opensles_convert_oboe_performance_mode(12, -1, -1, 0, 1, 2, 3),
            1
        );
        assert_eq!(
            oboe_rust_opensles_convert_oboe_performance_mode(12, 123, -1, 0, 1, 2, 3),
            2
        );
        assert_eq!(
            oboe_rust_opensles_convert_opensl_performance_mode(3, 0, 1, 2, 3, 10, 12, 11),
            11
        );
    }
}
