const UNSPECIFIED: i32 = 0;

const RESULT_OK: i32 = 0;
const RESULT_ERROR_DISCONNECTED: i32 = -899;
const RESULT_ERROR_INTERNAL: i32 = -896;
const RESULT_ERROR_INVALID_STATE: i32 = -895;
const RESULT_ERROR_CLOSED: i32 = -869;

const STREAM_STATE_CLOSED: i32 = 12;
const STREAM_STATE_DISCONNECTED: i32 = 13;

const DIRECTION_OUTPUT: i32 = 0;
const DIRECTION_INPUT: i32 = 1;

const AUDIO_API_UNSPECIFIED: i32 = UNSPECIFIED;
const AUDIO_API_OPENSLES: i32 = 1;
const AUDIO_API_AAUDIO: i32 = 2;

const PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;

const BACKEND_NONE: i32 = 0;
const BACKEND_AAUDIO: i32 = 1;
const BACKEND_OPENSLES_OUTPUT: i32 = 2;
const BACKEND_OPENSLES_INPUT: i32 = 3;

const CALLBACK_CONTINUE: i32 = 0;

#[no_mangle]
pub extern "C" fn oboe_rust_builder_will_use_aaudio(
    audio_api: i32,
    is_aaudio_supported: bool,
    is_aaudio_recommended: bool,
) -> bool {
    (audio_api == AUDIO_API_AAUDIO && is_aaudio_supported)
        || (audio_api == AUDIO_API_UNSPECIFIED && is_aaudio_recommended)
}

#[no_mangle]
pub extern "C" fn oboe_rust_builder_select_backend(
    audio_api: i32,
    direction: i32,
    is_aaudio_supported: bool,
    is_aaudio_recommended: bool,
) -> i32 {
    if is_aaudio_recommended && audio_api != AUDIO_API_OPENSLES {
        return BACKEND_AAUDIO;
    }
    if is_aaudio_supported && audio_api == AUDIO_API_AAUDIO {
        return BACKEND_AAUDIO;
    }
    match direction {
        DIRECTION_OUTPUT => BACKEND_OPENSLES_OUTPUT,
        DIRECTION_INPUT => BACKEND_OPENSLES_INPUT,
        _ => BACKEND_NONE,
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_builder_is_compatible(
    builder_sample_rate: i32,
    builder_format: i32,
    builder_frames_per_callback: i32,
    builder_channel_count: i32,
    stream_sample_rate: i32,
    stream_format: i32,
    stream_frames_per_callback: i32,
    stream_channel_count: i32,
) -> bool {
    (builder_sample_rate == UNSPECIFIED || builder_sample_rate == stream_sample_rate)
        && (builder_format == UNSPECIFIED || builder_format == stream_format)
        && (builder_frames_per_callback == UNSPECIFIED
            || builder_frames_per_callback == stream_frames_per_callback)
        && (builder_channel_count == UNSPECIFIED || builder_channel_count == stream_channel_count)
}

#[no_mangle]
pub extern "C" fn oboe_rust_stream_wait_transition_result(
    current_state: i32,
    starting_state: i32,
    ending_state: i32,
    wait_result: i32,
    next_state: i32,
) -> i32 {
    if current_state == STREAM_STATE_CLOSED {
        return RESULT_ERROR_CLOSED;
    }
    if current_state == STREAM_STATE_DISCONNECTED {
        return RESULT_ERROR_DISCONNECTED;
    }

    let observed_state = if current_state == starting_state && current_state != ending_state {
        if wait_result != RESULT_OK {
            return wait_result;
        }
        next_state
    } else {
        current_state
    };

    if observed_state == ending_state {
        RESULT_OK
    } else {
        RESULT_ERROR_INVALID_STATE
    }
}

#[no_mangle]
/// # Safety
///
/// `frames_available` must point to valid writable storage.
pub unsafe extern "C" fn oboe_rust_stream_available_frames(
    frames_read: i64,
    frames_written: i64,
    frames_available: *mut i32,
) -> i32 {
    if frames_available.is_null() {
        return RESULT_ERROR_INTERNAL;
    }
    if frames_read < 0 {
        return frames_read as i32;
    }
    if frames_written < 0 {
        return frames_written as i32;
    }
    *frames_available = (frames_written - frames_read) as i32;
    RESULT_OK
}

#[no_mangle]
pub extern "C" fn oboe_rust_stream_default_delay_before_close_millis(
    frames_per_burst: i32,
    sample_rate: i32,
    minimum: i32,
    maximum: i32,
) -> i32 {
    if sample_rate <= 0 {
        return minimum;
    }
    let delay = 1 + (frames_per_burst * 1000) / sample_rate;
    delay.clamp(minimum, maximum)
}

#[no_mangle]
pub extern "C" fn oboe_rust_stream_optimal_buffer_size(
    direction: i32,
    performance_mode: i32,
    buffer_capacity_in_frames: i32,
    frames_per_burst: i32,
    bursts_for_low_latency: i32,
) -> i32 {
    if direction == DIRECTION_INPUT {
        buffer_capacity_in_frames
    } else if direction == DIRECTION_OUTPUT && performance_mode == PERFORMANCE_MODE_LOW_LATENCY {
        frames_per_burst * bursts_for_low_latency
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_data_callback_should_continue(callback_result: i32) -> bool {
    callback_result == CALLBACK_CONTINUE
}
