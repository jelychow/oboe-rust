#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_adjust_input_capacity(
    capacity: i32,
    direction: i32,
    input_direction: i32,
    performance_mode: i32,
    low_latency_mode: i32,
    unspecified: i32,
    required_capacity_for_fast_track: i32,
    workarounds_enabled: bool,
) -> i32 {
    if workarounds_enabled
        && direction == input_direction
        && capacity != unspecified
        && capacity < required_capacity_for_fast_track
        && performance_mode == low_latency_mode
    {
        required_capacity_for_fast_track
    } else {
        capacity
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_session_performance_mode(
    performance_mode: i32,
    session_id: i32,
    session_id_none: i32,
    direction: i32,
    output_direction: i32,
    low_latency_mode: i32,
    none_mode: i32,
    workarounds_enabled: bool,
) -> i32 {
    if workarounds_enabled
        && session_id != session_id_none
        && direction == output_direction
        && performance_mode == low_latency_mode
    {
        none_mode
    } else {
        performance_mode
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_normalize_input_preset(
    input_preset: i32,
    sdk_version: i32,
    latest_unsupported_api: i32,
    voice_performance: i32,
    voice_recognition: i32,
) -> i32 {
    if sdk_version <= latest_unsupported_api && input_preset == voice_performance {
        voice_recognition
    } else {
        input_preset
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_spatialization_behavior(
    spatialization_behavior: i32,
    unspecified: i32,
    never: i32,
    setter_available: bool,
) -> i32 {
    if !setter_available || spatialization_behavior == unspecified {
        never
    } else {
        spatialization_behavior
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_coerce_open_result(
    open_result: i32,
    workarounds_enabled: bool,
    error_internal: i32,
) -> i32 {
    if open_result > 0 && workarounds_enabled {
        error_internal
    } else {
        open_result
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_force_starting_to_started(
    workarounds_enabled: bool,
    state: i32,
    starting_state: i32,
    started_state: i32,
) -> i32 {
    if workarounds_enabled && state == starting_state {
        started_state
    } else {
        state
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_request_already_satisfied(
    sdk_version: i32,
    android_api_o_mr1: i32,
    state: i32,
    first_terminal_state: i32,
    second_terminal_state: i32,
) -> bool {
    sdk_version <= android_api_o_mr1
        && (state == first_terminal_state || state == second_terminal_state)
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_calculate_latency_millis(
    is_output: bool,
    app_frame_index: i64,
    hardware_frame_index: i64,
    app_frame_app_time: i64,
    hardware_frame_hardware_time: i64,
    sample_rate: i32,
    nanos_per_second: i64,
    nanos_per_millisecond: i64,
) -> f64 {
    if sample_rate <= 0 || nanos_per_millisecond <= 0 {
        return 0.0;
    }
    let frame_index_delta = app_frame_index - hardware_frame_index;
    let frame_time_delta = (frame_index_delta * nanos_per_second) / sample_rate as i64;
    let app_frame_hardware_time = hardware_frame_hardware_time + frame_time_delta;
    let latency_nanos = if is_output {
        app_frame_hardware_time - app_frame_app_time
    } else {
        app_frame_app_time - app_frame_hardware_time
    };
    latency_nanos as f64 / nanos_per_millisecond as f64
}

#[cfg(test)]
mod aaudio_output_tests {
    use super::*;
    use core::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicPtr, Ordering};
    use std::sync::Mutex;

    const BUILDER_PTR: *mut c_void = 0x1000 as *mut c_void;
    const STREAM_PTR: *mut c_void = 0x2000 as *mut c_void;
    const RESULT_OK: i32 = 0;
    const RESULT_ERROR_CLOSED: i32 = -869;
    const DIRECTION_INPUT: i32 = 1;
    const DIRECTION_OUTPUT: i32 = 0;
    const FORMAT_I16: i32 = 1;
    const INPUT_PRESET_VOICE_COMMUNICATION: i32 = 6;
    const PRIVACY_SENSITIVE_UNSPECIFIED: i32 = 0;
    const PRIVACY_SENSITIVE_ENABLED: i32 = 1;
    const PRIVACY_SENSITIVE_DISABLED: i32 = 2;
    const SHARING_MODE_SHARED: i32 = 1;
    const PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;
    const STREAM_STATE_OPEN: i32 = 2;
    const STREAM_STATE_STARTED: i32 = 4;
    const STREAM_STATE_STOPPED: i32 = 10;

    static CREATED_BUILDERS: AtomicI32 = AtomicI32::new(0);
    static DELETED_BUILDERS: AtomicI32 = AtomicI32::new(0);
    static OPENED_STREAMS: AtomicI32 = AtomicI32::new(0);
    static CLOSED_STREAMS: AtomicI32 = AtomicI32::new(0);
    static RELEASED_STREAMS: AtomicI32 = AtomicI32::new(0);
    static START_REQUESTS: AtomicI32 = AtomicI32::new(0);
    static STOP_REQUESTS: AtomicI32 = AtomicI32::new(0);
    static OFFLOAD_DELAY_PADDING_SET: AtomicI32 = AtomicI32::new(0);
    static OFFLOAD_EOS_SET: AtomicI32 = AtomicI32::new(0);
    static FLUSH_FROM_FRAME_CALLED: AtomicI32 = AtomicI32::new(0);
    static PLAYBACK_PARAMETERS_SET: AtomicI32 = AtomicI32::new(0);
    static PLAYBACK_PARAMETERS_GET: AtomicI32 = AtomicI32::new(0);
    static READ_FRAMES: AtomicI32 = AtomicI32::new(0);
    static WRITTEN_FRAMES: AtomicI32 = AtomicI32::new(0);
    static LAST_DIRECTION: AtomicI32 = AtomicI32::new(-1);
    static LAST_CHANNEL_COUNT: AtomicI32 = AtomicI32::new(-1);
    static LAST_FORMAT: AtomicI32 = AtomicI32::new(-1);
    static LAST_INPUT_PRESET: AtomicI32 = AtomicI32::new(-1);
    static LAST_SAMPLE_RATE: AtomicI32 = AtomicI32::new(-1);
    static LAST_BUFFER_CAPACITY: AtomicI32 = AtomicI32::new(-1);
    static LAST_FRAMES_PER_CALLBACK: AtomicI32 = AtomicI32::new(-1);
    static LAST_CALLBACK_USER_DATA: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());
    static STATE: AtomicI32 = AtomicI32::new(STREAM_STATE_OPEN);
    static FRAMES_READ: AtomicI64 = AtomicI64::new(0);
    static FRAMES_WRITTEN: AtomicI64 = AtomicI64::new(0);
    static DATA_CALLBACK_SET: AtomicBool = AtomicBool::new(false);
    static ERROR_CALLBACK_SET: AtomicBool = AtomicBool::new(false);
    static PARTIAL_DATA_CALLBACK_SET: AtomicBool = AtomicBool::new(false);
    static PRESENTATION_CALLBACK_SET: AtomicBool = AtomicBool::new(false);
    static ROUTING_CALLBACK_SET: AtomicBool = AtomicBool::new(false);
    static PRIVACY_SENSITIVE_SET: AtomicBool = AtomicBool::new(false);
    static PRIVACY_SENSITIVE_VALUE: AtomicBool = AtomicBool::new(false);
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_fake_platform() {
        CREATED_BUILDERS.store(0, Ordering::SeqCst);
        DELETED_BUILDERS.store(0, Ordering::SeqCst);
        OPENED_STREAMS.store(0, Ordering::SeqCst);
        CLOSED_STREAMS.store(0, Ordering::SeqCst);
        RELEASED_STREAMS.store(0, Ordering::SeqCst);
        START_REQUESTS.store(0, Ordering::SeqCst);
        STOP_REQUESTS.store(0, Ordering::SeqCst);
        OFFLOAD_DELAY_PADDING_SET.store(0, Ordering::SeqCst);
        OFFLOAD_EOS_SET.store(0, Ordering::SeqCst);
        FLUSH_FROM_FRAME_CALLED.store(0, Ordering::SeqCst);
        PLAYBACK_PARAMETERS_SET.store(0, Ordering::SeqCst);
        PLAYBACK_PARAMETERS_GET.store(0, Ordering::SeqCst);
        READ_FRAMES.store(0, Ordering::SeqCst);
        WRITTEN_FRAMES.store(0, Ordering::SeqCst);
        LAST_DIRECTION.store(-1, Ordering::SeqCst);
        LAST_CHANNEL_COUNT.store(-1, Ordering::SeqCst);
        LAST_FORMAT.store(-1, Ordering::SeqCst);
        LAST_INPUT_PRESET.store(-1, Ordering::SeqCst);
        LAST_SAMPLE_RATE.store(-1, Ordering::SeqCst);
        LAST_BUFFER_CAPACITY.store(-1, Ordering::SeqCst);
        LAST_FRAMES_PER_CALLBACK.store(-1, Ordering::SeqCst);
        LAST_CALLBACK_USER_DATA.store(core::ptr::null_mut(), Ordering::SeqCst);
        STATE.store(STREAM_STATE_OPEN, Ordering::SeqCst);
        FRAMES_READ.store(0, Ordering::SeqCst);
        FRAMES_WRITTEN.store(0, Ordering::SeqCst);
        DATA_CALLBACK_SET.store(false, Ordering::SeqCst);
        ERROR_CALLBACK_SET.store(false, Ordering::SeqCst);
        PARTIAL_DATA_CALLBACK_SET.store(false, Ordering::SeqCst);
        PRESENTATION_CALLBACK_SET.store(false, Ordering::SeqCst);
        ROUTING_CALLBACK_SET.store(false, Ordering::SeqCst);
        PRIVACY_SENSITIVE_SET.store(false, Ordering::SeqCst);
        PRIVACY_SENSITIVE_VALUE.store(false, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_create_stream_builder(builder: *mut *mut c_void) -> i32 {
        CREATED_BUILDERS.fetch_add(1, Ordering::SeqCst);
        *builder = BUILDER_PTR;
        RESULT_OK
    }

    unsafe extern "C" fn fake_builder_open_stream(
        _builder: *mut c_void,
        stream: *mut *mut c_void,
    ) -> i32 {
        OPENED_STREAMS.fetch_add(1, Ordering::SeqCst);
        *stream = STREAM_PTR;
        RESULT_OK
    }

    unsafe extern "C" fn fake_builder_delete(_builder: *mut c_void) -> i32 {
        DELETED_BUILDERS.fetch_add(1, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_set_buffer_capacity(_builder: *mut c_void, value: i32) {
        LAST_BUFFER_CAPACITY.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_channel_count(_builder: *mut c_void, value: i32) {
        LAST_CHANNEL_COUNT.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_ignored_i32(_builder: *mut c_void, _value: i32) {}

    unsafe extern "C" fn fake_set_input_preset(_builder: *mut c_void, value: i32) {
        LAST_INPUT_PRESET.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_privacy_sensitive(_builder: *mut c_void, value: bool) {
        PRIVACY_SENSITIVE_SET.store(true, Ordering::SeqCst);
        PRIVACY_SENSITIVE_VALUE.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_direction(_builder: *mut c_void, value: i32) {
        LAST_DIRECTION.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_format(_builder: *mut c_void, value: i32) {
        LAST_FORMAT.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_frames_per_callback(_builder: *mut c_void, value: i32) {
        LAST_FRAMES_PER_CALLBACK.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_sample_rate(_builder: *mut c_void, value: i32) {
        LAST_SAMPLE_RATE.store(value, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_data_callback(
        _builder: *mut c_void,
        _callback: OboeRustAAudioDataCallback,
        user_data: *mut c_void,
    ) {
        DATA_CALLBACK_SET.store(true, Ordering::SeqCst);
        LAST_CALLBACK_USER_DATA.store(user_data, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_error_callback(
        _builder: *mut c_void,
        _callback: OboeRustAAudioErrorCallback,
        user_data: *mut c_void,
    ) {
        ERROR_CALLBACK_SET.store(true, Ordering::SeqCst);
        LAST_CALLBACK_USER_DATA.store(user_data, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_partial_data_callback(
        _builder: *mut c_void,
        _callback: OboeRustAAudioPartialDataCallback,
        user_data: *mut c_void,
    ) {
        PARTIAL_DATA_CALLBACK_SET.store(true, Ordering::SeqCst);
        LAST_CALLBACK_USER_DATA.store(user_data, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_presentation_callback(
        _builder: *mut c_void,
        _callback: OboeRustAAudioPresentationCallback,
        user_data: *mut c_void,
    ) {
        PRESENTATION_CALLBACK_SET.store(true, Ordering::SeqCst);
        LAST_CALLBACK_USER_DATA.store(user_data, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_set_routing_callback(
        _builder: *mut c_void,
        _callback: OboeRustAAudioRoutingChangedCallback,
        user_data: *mut c_void,
    ) {
        ROUTING_CALLBACK_SET.store(true, Ordering::SeqCst);
        LAST_CALLBACK_USER_DATA.store(user_data, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_close(_stream: *mut c_void) -> i32 {
        CLOSED_STREAMS.fetch_add(1, Ordering::SeqCst);
        STATE.store(12, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_release(_stream: *mut c_void) -> i32 {
        RELEASED_STREAMS.fetch_add(1, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_request_start(_stream: *mut c_void) -> i32 {
        START_REQUESTS.fetch_add(1, Ordering::SeqCst);
        STATE.store(STREAM_STATE_STARTED, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_request_stop(_stream: *mut c_void) -> i32 {
        STOP_REQUESTS.fetch_add(1, Ordering::SeqCst);
        STATE.store(STREAM_STATE_STOPPED, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_write(
        _stream: *mut c_void,
        _buffer: *const c_void,
        num_frames: i32,
        _timeout_nanoseconds: i64,
    ) -> i32 {
        WRITTEN_FRAMES.fetch_add(num_frames, Ordering::SeqCst);
        FRAMES_WRITTEN.fetch_add(num_frames as i64, Ordering::SeqCst);
        num_frames
    }

    unsafe extern "C" fn fake_read(
        _stream: *mut c_void,
        _buffer: *mut c_void,
        num_frames: i32,
        _timeout_nanoseconds: i64,
    ) -> i32 {
        READ_FRAMES.fetch_add(num_frames, Ordering::SeqCst);
        FRAMES_READ.fetch_add(num_frames as i64, Ordering::SeqCst);
        num_frames
    }

    unsafe extern "C" fn fake_get_i32(_stream: *mut c_void) -> i32 {
        2
    }

    unsafe extern "C" fn fake_get_channel_count(_stream: *mut c_void) -> i32 {
        LAST_CHANNEL_COUNT.load(Ordering::SeqCst)
    }

    unsafe extern "C" fn fake_get_sample_rate(_stream: *mut c_void) -> i32 {
        48000
    }

    unsafe extern "C" fn fake_get_buffer_capacity(_stream: *mut c_void) -> i32 {
        960
    }

    unsafe extern "C" fn fake_get_buffer_size(_stream: *mut c_void) -> i32 {
        384
    }

    unsafe extern "C" fn fake_get_frames_per_burst(_stream: *mut c_void) -> i32 {
        192
    }

    unsafe extern "C" fn fake_get_state(_stream: *mut c_void) -> i32 {
        STATE.load(Ordering::SeqCst)
    }

    unsafe extern "C" fn fake_get_frames_written(_stream: *mut c_void) -> i64 {
        FRAMES_WRITTEN.load(Ordering::SeqCst)
    }

    unsafe extern "C" fn fake_get_input_preset(_stream: *mut c_void) -> i32 {
        LAST_INPUT_PRESET.load(Ordering::SeqCst)
    }

    unsafe extern "C" fn fake_get_frames_read(_stream: *mut c_void) -> i64 {
        FRAMES_READ.load(Ordering::SeqCst)
    }

    unsafe extern "C" fn fake_set_offload_delay_padding(
        _stream: *mut c_void,
        _delay_in_frames: i32,
        _padding_in_frames: i32,
    ) -> i32 {
        OFFLOAD_DELAY_PADDING_SET.fetch_add(1, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_get_offload_delay(_stream: *mut c_void) -> i32 {
        12
    }

    unsafe extern "C" fn fake_get_offload_padding(_stream: *mut c_void) -> i32 {
        34
    }

    unsafe extern "C" fn fake_set_offload_end_of_stream(_stream: *mut c_void) -> i32 {
        OFFLOAD_EOS_SET.fetch_add(1, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_flush_from_frame(
        _stream: *mut c_void,
        _accuracy: i32,
        position_in_frames: *mut i64,
    ) -> i32 {
        FLUSH_FROM_FRAME_CALLED.fetch_add(1, Ordering::SeqCst);
        *position_in_frames = 1234;
        RESULT_OK
    }

    unsafe extern "C" fn fake_get_playback_parameters(
        _stream: *mut c_void,
        parameters: *mut OboeRustAAudioPlaybackParameters,
    ) -> i32 {
        PLAYBACK_PARAMETERS_GET.fetch_add(1, Ordering::SeqCst);
        *parameters = OboeRustAAudioPlaybackParameters {
            fallback_mode: 1,
            stretch_mode: 1,
            pitch: 1.25,
            speed: 0.75,
        };
        RESULT_OK
    }

    unsafe extern "C" fn fake_set_playback_parameters(
        _stream: *mut c_void,
        parameters: *const OboeRustAAudioPlaybackParameters,
    ) -> i32 {
        assert_eq!((*parameters).fallback_mode, 1);
        assert_eq!((*parameters).stretch_mode, 1);
        PLAYBACK_PARAMETERS_SET.fetch_add(1, Ordering::SeqCst);
        RESULT_OK
    }

    unsafe extern "C" fn fake_is_privacy_sensitive(_stream: *mut c_void) -> bool {
        PRIVACY_SENSITIVE_VALUE.load(Ordering::SeqCst)
    }

    fn fake_platform() -> OboeRustAAudioPlatform {
        OboeRustAAudioPlatform {
            create_stream_builder: Some(fake_create_stream_builder),
            builder_open_stream: Some(fake_builder_open_stream),
            builder_delete: Some(fake_builder_delete),
            builder_set_buffer_capacity_in_frames: Some(fake_set_buffer_capacity),
            builder_set_channel_count: Some(fake_set_channel_count),
            builder_set_device_id: Some(fake_set_ignored_i32),
            builder_set_direction: Some(fake_set_direction),
            builder_set_format: Some(fake_set_format),
            builder_set_frames_per_data_callback: Some(fake_set_frames_per_callback),
            builder_set_input_preset: Some(fake_set_input_preset),
            builder_set_performance_mode: Some(fake_set_ignored_i32),
            builder_set_privacy_sensitive: Some(fake_set_privacy_sensitive),
            builder_set_sample_rate: Some(fake_set_sample_rate),
            builder_set_sharing_mode: Some(fake_set_ignored_i32),
            builder_set_data_callback: Some(fake_set_data_callback),
            builder_set_error_callback: Some(fake_set_error_callback),
            builder_set_partial_data_callback: Some(fake_set_partial_data_callback),
            builder_set_presentation_end_callback: Some(fake_set_presentation_callback),
            builder_set_routing_changed_callback: Some(fake_set_routing_callback),
            stream_close: Some(fake_close),
            stream_release: Some(fake_release),
            stream_request_start: Some(fake_request_start),
            stream_request_stop: Some(fake_request_stop),
            stream_read: Some(fake_read),
            stream_write: Some(fake_write),
            stream_get_channel_count: Some(fake_get_channel_count),
            stream_get_format: Some(fake_get_i32),
            stream_get_input_preset: Some(fake_get_input_preset),
            stream_get_sample_rate: Some(fake_get_sample_rate),
            stream_get_sharing_mode: Some(fake_get_i32),
            stream_get_performance_mode: Some(fake_get_i32),
            stream_get_buffer_capacity: Some(fake_get_buffer_capacity),
            stream_get_buffer_size: Some(fake_get_buffer_size),
            stream_get_frames_per_burst: Some(fake_get_frames_per_burst),
            stream_get_state: Some(fake_get_state),
            stream_get_frames_read: Some(fake_get_frames_read),
            stream_get_frames_written: Some(fake_get_frames_written),
            stream_is_privacy_sensitive: Some(fake_is_privacy_sensitive),
            stream_set_offload_delay_padding: Some(fake_set_offload_delay_padding),
            stream_get_offload_delay: Some(fake_get_offload_delay),
            stream_get_offload_padding: Some(fake_get_offload_padding),
            stream_set_offload_end_of_stream: Some(fake_set_offload_end_of_stream),
            stream_flush_from_frame: Some(fake_flush_from_frame),
            stream_get_playback_parameters: Some(fake_get_playback_parameters),
            stream_set_playback_parameters: Some(fake_set_playback_parameters),
            ..OboeRustAAudioPlatform::empty()
        }
    }

    extern "C" fn fake_data_callback(
        _stream: *mut c_void,
        _user_data: *mut c_void,
        _audio_data: *mut c_void,
        _num_frames: i32,
    ) -> i32 {
        0
    }

    extern "C" fn fake_error_callback(_stream: *mut c_void, _user_data: *mut c_void, _error: i32) {}

    extern "C" fn fake_partial_data_callback(
        _stream: *mut c_void,
        _user_data: *mut c_void,
        _audio_data: *mut c_void,
        _num_frames: i32,
    ) -> i32 {
        0
    }

    extern "C" fn fake_presentation_callback(_stream: *mut c_void, _user_data: *mut c_void) {}

    extern "C" fn fake_routing_callback(
        _stream: *mut c_void,
        _user_data: *mut c_void,
        _device_ids: *const i32,
        _device_count: i32,
    ) {
    }

    fn output_settings() -> OboeRustAAudioOutputSettings {
        OboeRustAAudioOutputSettings {
            direction: DIRECTION_OUTPUT,
            device_id: 0,
            sample_rate: 48000,
            channel_count: 2,
            channel_mask: 0,
            format: FORMAT_I16,
            sharing_mode: SHARING_MODE_SHARED,
            performance_mode: PERFORMANCE_MODE_LOW_LATENCY,
            buffer_capacity_in_frames: 960,
            frames_per_data_callback: 192,
            session_id: 0,
            usage: 1,
            content_type: 2,
            allowed_capture_policy: 0,
            is_content_spatialized: false,
            spatialization_behavior: 0,
            package_name: core::ptr::null(),
            attribution_tag: core::ptr::null(),
            data_callback: Some(fake_data_callback),
            error_callback: Some(fake_error_callback),
            partial_data_callback: None,
            presentation_end_callback: None,
            routing_changed_callback: None,
            user_data: 0x4444 as *mut c_void,
        }
    }

    fn input_settings() -> OboeRustAAudioInputSettings {
        OboeRustAAudioInputSettings {
            direction: DIRECTION_INPUT,
            device_id: 0,
            sample_rate: 48000,
            channel_count: 1,
            channel_mask: 0,
            format: FORMAT_I16,
            sharing_mode: SHARING_MODE_SHARED,
            performance_mode: PERFORMANCE_MODE_LOW_LATENCY,
            buffer_capacity_in_frames: 4096,
            frames_per_data_callback: 192,
            input_preset: INPUT_PRESET_VOICE_COMMUNICATION,
            privacy_sensitive_mode: PRIVACY_SENSITIVE_ENABLED,
            privacy_sensitive_mode_unspecified: PRIVACY_SENSITIVE_UNSPECIFIED,
            privacy_sensitive_mode_enabled: PRIVACY_SENSITIVE_ENABLED,
            privacy_sensitive_mode_disabled: PRIVACY_SENSITIVE_DISABLED,
            session_id: 0,
            usage: 0,
            content_type: 0,
            allowed_capture_policy: 0,
            package_name: core::ptr::null(),
            attribution_tag: core::ptr::null(),
            is_content_spatialized: false,
            spatialization_behavior: 0,
            data_callback: Some(fake_data_callback),
            error_callback: Some(fake_error_callback),
            partial_data_callback: None,
            presentation_end_callback: None,
            routing_changed_callback: None,
            user_data: 0x5555 as *mut c_void,
        }
    }

    #[test]
    fn aaudio_output_open_is_owned_by_rust_and_caches_properties() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_fake_platform();
        let platform = fake_platform();
        let settings = output_settings();
        let mut properties = OboeRustAAudioOutputProperties::default();

        let stream = unsafe { oboe_rust_aaudio_output_open(&platform, &settings, &mut properties) };

        assert!(!stream.is_null());
        assert_eq!(CREATED_BUILDERS.load(Ordering::SeqCst), 1);
        assert_eq!(OPENED_STREAMS.load(Ordering::SeqCst), 1);
        assert_eq!(DELETED_BUILDERS.load(Ordering::SeqCst), 1);
        assert_eq!(LAST_DIRECTION.load(Ordering::SeqCst), DIRECTION_OUTPUT);
        assert_eq!(LAST_FORMAT.load(Ordering::SeqCst), FORMAT_I16);
        assert_eq!(LAST_CHANNEL_COUNT.load(Ordering::SeqCst), 2);
        assert_eq!(LAST_SAMPLE_RATE.load(Ordering::SeqCst), 48000);
        assert_eq!(LAST_BUFFER_CAPACITY.load(Ordering::SeqCst), 960);
        assert_eq!(LAST_FRAMES_PER_CALLBACK.load(Ordering::SeqCst), 192);
        assert!(DATA_CALLBACK_SET.load(Ordering::SeqCst));
        assert!(ERROR_CALLBACK_SET.load(Ordering::SeqCst));
        assert_eq!(
            LAST_CALLBACK_USER_DATA.load(Ordering::SeqCst),
            settings.user_data
        );
        assert_eq!(properties.raw_stream, STREAM_PTR);
        assert_eq!(properties.channel_count, 2);
        assert_eq!(properties.sample_rate, 48000);
        assert_eq!(properties.buffer_capacity_in_frames, 960);
        assert_eq!(properties.buffer_size_in_frames, 384);
        assert_eq!(properties.frames_per_burst, 192);

        unsafe {
            assert_eq!(oboe_rust_aaudio_output_destroy(stream), RESULT_OK);
        }
    }

    #[test]
    fn aaudio_output_lifecycle_calls_platform_and_returns_closed_after_close() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_fake_platform();
        let platform = fake_platform();
        let settings = output_settings();
        let mut properties = OboeRustAAudioOutputProperties::default();
        let stream = unsafe { oboe_rust_aaudio_output_open(&platform, &settings, &mut properties) };

        assert_eq!(
            unsafe { oboe_rust_aaudio_output_request_start(stream) },
            RESULT_OK
        );
        assert_eq!(START_REQUESTS.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_get_state(stream) },
            STREAM_STATE_STARTED
        );

        let frames = [1_i16, 2, 3, 4];
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_write(stream, frames.as_ptr().cast(), 4, 0,) },
            4
        );
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_get_frames_written(stream) },
            4
        );

        assert_eq!(
            unsafe { oboe_rust_aaudio_output_request_stop(stream) },
            RESULT_OK
        );
        assert_eq!(STOP_REQUESTS.load(Ordering::SeqCst), 1);
        assert_eq!(unsafe { oboe_rust_aaudio_output_close(stream) }, RESULT_OK);
        assert_eq!(CLOSED_STREAMS.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_request_start(stream) },
            RESULT_ERROR_CLOSED
        );
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_write(stream, frames.as_ptr().cast(), 4, 0) },
            RESULT_ERROR_CLOSED
        );

        unsafe {
            assert_eq!(oboe_rust_aaudio_output_destroy(stream), RESULT_OK);
        }
    }

    #[test]
    fn aaudio_output_registers_partial_presentation_and_routing_callbacks() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_fake_platform();
        let platform = fake_platform();
        let mut settings = output_settings();
        settings.data_callback = None;
        settings.partial_data_callback = Some(fake_partial_data_callback);
        settings.presentation_end_callback = Some(fake_presentation_callback);
        settings.routing_changed_callback = Some(fake_routing_callback);
        let mut properties = OboeRustAAudioOutputProperties::default();

        let stream = unsafe { oboe_rust_aaudio_output_open(&platform, &settings, &mut properties) };

        assert!(!stream.is_null());
        assert!(!DATA_CALLBACK_SET.load(Ordering::SeqCst));
        assert!(PARTIAL_DATA_CALLBACK_SET.load(Ordering::SeqCst));
        assert!(PRESENTATION_CALLBACK_SET.load(Ordering::SeqCst));
        assert!(ROUTING_CALLBACK_SET.load(Ordering::SeqCst));
        assert_eq!(
            LAST_CALLBACK_USER_DATA.load(Ordering::SeqCst),
            settings.user_data
        );

        unsafe {
            assert_eq!(oboe_rust_aaudio_output_destroy(stream), RESULT_OK);
        }
    }

    #[test]
    fn aaudio_output_extension_calls_are_forwarded_to_platform() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_fake_platform();
        let platform = fake_platform();
        let settings = output_settings();
        let mut properties = OboeRustAAudioOutputProperties::default();
        let stream = unsafe { oboe_rust_aaudio_output_open(&platform, &settings, &mut properties) };

        assert_eq!(
            unsafe { oboe_rust_aaudio_output_set_offload_delay_padding(stream, 10, 20) },
            RESULT_OK
        );
        assert_eq!(OFFLOAD_DELAY_PADDING_SET.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_get_offload_delay(stream) },
            12
        );
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_get_offload_padding(stream) },
            34
        );
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_set_offload_end_of_stream(stream) },
            RESULT_OK
        );
        assert_eq!(OFFLOAD_EOS_SET.load(Ordering::SeqCst), 1);

        let mut frame_position = 0_i64;
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_flush_from_frame(stream, 7, &mut frame_position) },
            RESULT_OK
        );
        assert_eq!(FLUSH_FROM_FRAME_CALLED.load(Ordering::SeqCst), 1);
        assert_eq!(frame_position, 1234);

        let mut playback_parameters = OboeRustAAudioPlaybackParameters {
            fallback_mode: 0,
            stretch_mode: 0,
            pitch: 0.0,
            speed: 0.0,
        };
        assert_eq!(
            unsafe {
                oboe_rust_aaudio_output_get_playback_parameters(stream, &mut playback_parameters)
            },
            RESULT_OK
        );
        assert_eq!(PLAYBACK_PARAMETERS_GET.load(Ordering::SeqCst), 1);
        assert_eq!(playback_parameters.fallback_mode, 1);
        assert_eq!(playback_parameters.stretch_mode, 1);
        assert_eq!(playback_parameters.pitch, 1.25);
        assert_eq!(playback_parameters.speed, 0.75);
        assert_eq!(
            unsafe {
                oboe_rust_aaudio_output_set_playback_parameters(stream, &playback_parameters)
            },
            RESULT_OK
        );
        assert_eq!(PLAYBACK_PARAMETERS_SET.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_aaudio_output_release(stream) },
            RESULT_OK
        );
        assert_eq!(RELEASED_STREAMS.load(Ordering::SeqCst), 1);

        unsafe {
            assert_eq!(oboe_rust_aaudio_output_destroy(stream), RESULT_OK);
        }
    }

    #[test]
    fn aaudio_input_open_is_owned_by_rust_and_caches_input_properties() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_fake_platform();
        let platform = fake_platform();
        let settings = input_settings();
        let mut properties = OboeRustAAudioInputProperties::default();

        let stream = unsafe { oboe_rust_aaudio_input_open(&platform, &settings, &mut properties) };

        assert!(!stream.is_null());
        assert_eq!(CREATED_BUILDERS.load(Ordering::SeqCst), 1);
        assert_eq!(OPENED_STREAMS.load(Ordering::SeqCst), 1);
        assert_eq!(DELETED_BUILDERS.load(Ordering::SeqCst), 1);
        assert_eq!(LAST_DIRECTION.load(Ordering::SeqCst), DIRECTION_INPUT);
        assert_eq!(
            LAST_INPUT_PRESET.load(Ordering::SeqCst),
            INPUT_PRESET_VOICE_COMMUNICATION
        );
        assert!(PRIVACY_SENSITIVE_SET.load(Ordering::SeqCst));
        assert!(PRIVACY_SENSITIVE_VALUE.load(Ordering::SeqCst));
        assert_eq!(properties.raw_stream, STREAM_PTR);
        assert_eq!(properties.channel_count, 1);
        assert_eq!(properties.sample_rate, 48000);
        assert_eq!(properties.input_preset, INPUT_PRESET_VOICE_COMMUNICATION);
        assert_eq!(properties.privacy_sensitive_mode, PRIVACY_SENSITIVE_ENABLED);
        assert_eq!(properties.buffer_capacity_in_frames, 960);
        assert_eq!(properties.frames_per_burst, 192);

        unsafe {
            assert_eq!(oboe_rust_aaudio_input_destroy(stream), RESULT_OK);
        }
    }

    #[test]
    fn aaudio_input_lifecycle_reads_and_returns_closed_after_close() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_fake_platform();
        let platform = fake_platform();
        let settings = input_settings();
        let mut properties = OboeRustAAudioInputProperties::default();
        let stream = unsafe { oboe_rust_aaudio_input_open(&platform, &settings, &mut properties) };

        assert_eq!(
            unsafe { oboe_rust_aaudio_input_request_start(stream) },
            RESULT_OK
        );
        assert_eq!(START_REQUESTS.load(Ordering::SeqCst), 1);

        let mut frames = [0_i16; 4];
        assert_eq!(
            unsafe { oboe_rust_aaudio_input_read(stream, frames.as_mut_ptr().cast(), 4, 0,) },
            4
        );
        assert_eq!(unsafe { oboe_rust_aaudio_input_get_frames_read(stream) }, 4);

        assert_eq!(
            unsafe { oboe_rust_aaudio_input_request_stop(stream) },
            RESULT_OK
        );
        assert_eq!(STOP_REQUESTS.load(Ordering::SeqCst), 1);
        assert_eq!(unsafe { oboe_rust_aaudio_input_close(stream) }, RESULT_OK);
        assert_eq!(CLOSED_STREAMS.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_aaudio_input_request_start(stream) },
            RESULT_ERROR_CLOSED
        );
        assert_eq!(
            unsafe { oboe_rust_aaudio_input_read(stream, frames.as_mut_ptr().cast(), 4, 0) },
            RESULT_ERROR_CLOSED
        );

        unsafe {
            assert_eq!(oboe_rust_aaudio_input_destroy(stream), RESULT_OK);
        }
    }
}
use core::ffi::{c_char, c_void};
use core::mem;
use core::ptr;

const RESULT_OK: i32 = 0;
const RESULT_ERROR_CLOSED: i32 = -869;
const RESULT_ERROR_NULL: i32 = -886;
const RESULT_ERROR_UNIMPLEMENTED: i32 = -890;
const RESULT_ERROR_INTERNAL: i32 = -896;

const STREAM_STATE_UNKNOWN: i32 = 1;
const STREAM_STATE_CLOSED: i32 = 12;

#[cfg(not(test))]
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

#[cfg(test)]
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

pub type OboeRustAAudioDataCallback =
    Option<extern "C" fn(*mut c_void, *mut c_void, *mut c_void, i32) -> i32>;
pub type OboeRustAAudioErrorCallback = Option<extern "C" fn(*mut c_void, *mut c_void, i32)>;
pub type OboeRustAAudioPartialDataCallback =
    Option<extern "C" fn(*mut c_void, *mut c_void, *mut c_void, i32) -> i32>;
pub type OboeRustAAudioPresentationCallback = Option<extern "C" fn(*mut c_void, *mut c_void)>;
pub type OboeRustAAudioRoutingChangedCallback =
    Option<extern "C" fn(*mut c_void, *mut c_void, *const i32, i32)>;

#[repr(C)]
pub struct OboeRustAAudioOutputStream {
    _private: [u8; 0],
}

#[repr(C)]
pub struct OboeRustAAudioInputStream {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OboeRustAAudioPlaybackParameters {
    pub fallback_mode: i32,
    pub stretch_mode: i32,
    pub pitch: f32,
    pub speed: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustAAudioPlatform {
    pub create_stream_builder: Option<unsafe extern "C" fn(*mut *mut c_void) -> i32>,
    pub builder_open_stream: Option<unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32>,
    pub builder_delete: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub builder_set_buffer_capacity_in_frames: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_channel_count: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_device_id: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_direction: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_format: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_frames_per_data_callback: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_performance_mode: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_sample_rate: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_sharing_mode: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_channel_mask: Option<unsafe extern "C" fn(*mut c_void, u32)>,
    pub builder_set_usage: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_content_type: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_input_preset: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_session_id: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_privacy_sensitive: Option<unsafe extern "C" fn(*mut c_void, bool)>,
    pub builder_set_allowed_capture_policy: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_package_name: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
    pub builder_set_attribution_tag: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
    pub builder_set_is_content_spatialized: Option<unsafe extern "C" fn(*mut c_void, bool)>,
    pub builder_set_spatialization_behavior: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub builder_set_data_callback:
        Option<unsafe extern "C" fn(*mut c_void, OboeRustAAudioDataCallback, *mut c_void)>,
    pub builder_set_error_callback:
        Option<unsafe extern "C" fn(*mut c_void, OboeRustAAudioErrorCallback, *mut c_void)>,
    pub builder_set_partial_data_callback:
        Option<unsafe extern "C" fn(*mut c_void, OboeRustAAudioPartialDataCallback, *mut c_void)>,
    pub builder_set_presentation_end_callback:
        Option<unsafe extern "C" fn(*mut c_void, OboeRustAAudioPresentationCallback, *mut c_void)>,
    pub builder_set_routing_changed_callback: Option<
        unsafe extern "C" fn(*mut c_void, OboeRustAAudioRoutingChangedCallback, *mut c_void),
    >,
    pub stream_close: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_release: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_request_start: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_request_pause: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_request_flush: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_request_stop: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_write: Option<unsafe extern "C" fn(*mut c_void, *const c_void, i32, i64) -> i32>,
    pub stream_read: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, i32, i64) -> i32>,
    pub stream_wait_for_state_change:
        Option<unsafe extern "C" fn(*mut c_void, i32, *mut i32, i64) -> i32>,
    pub stream_get_timestamp:
        Option<unsafe extern "C" fn(*mut c_void, i32, *mut i64, *mut i64) -> i32>,
    pub stream_set_buffer_size: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub stream_get_channel_count: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_device_id: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_format: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_sample_rate: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_sharing_mode: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_performance_mode: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_buffer_capacity: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_buffer_size: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_frames_per_burst: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_state: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_xrun_count: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_frames_read: Option<unsafe extern "C" fn(*mut c_void) -> i64>,
    pub stream_get_frames_written: Option<unsafe extern "C" fn(*mut c_void) -> i64>,
    pub stream_get_usage: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_content_type: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_input_preset: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_session_id: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_is_privacy_sensitive: Option<unsafe extern "C" fn(*mut c_void) -> bool>,
    pub stream_get_allowed_capture_policy: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_channel_mask: Option<unsafe extern "C" fn(*mut c_void) -> u32>,
    pub stream_is_content_spatialized: Option<unsafe extern "C" fn(*mut c_void) -> bool>,
    pub stream_get_spatialization_behavior: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_hardware_channel_count: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_hardware_sample_rate: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_hardware_format: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_set_offload_delay_padding:
        Option<unsafe extern "C" fn(*mut c_void, i32, i32) -> i32>,
    pub stream_get_offload_delay: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_get_offload_padding: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_set_offload_end_of_stream: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub stream_flush_from_frame: Option<unsafe extern "C" fn(*mut c_void, i32, *mut i64) -> i32>,
    pub stream_get_playback_parameters:
        Option<unsafe extern "C" fn(*mut c_void, *mut OboeRustAAudioPlaybackParameters) -> i32>,
    pub stream_set_playback_parameters:
        Option<unsafe extern "C" fn(*mut c_void, *const OboeRustAAudioPlaybackParameters) -> i32>,
}

impl OboeRustAAudioPlatform {
    #[cfg(test)]
    pub const fn empty() -> Self {
        Self {
            create_stream_builder: None,
            builder_open_stream: None,
            builder_delete: None,
            builder_set_buffer_capacity_in_frames: None,
            builder_set_channel_count: None,
            builder_set_device_id: None,
            builder_set_direction: None,
            builder_set_format: None,
            builder_set_frames_per_data_callback: None,
            builder_set_performance_mode: None,
            builder_set_sample_rate: None,
            builder_set_sharing_mode: None,
            builder_set_channel_mask: None,
            builder_set_usage: None,
            builder_set_content_type: None,
            builder_set_input_preset: None,
            builder_set_session_id: None,
            builder_set_privacy_sensitive: None,
            builder_set_allowed_capture_policy: None,
            builder_set_package_name: None,
            builder_set_attribution_tag: None,
            builder_set_is_content_spatialized: None,
            builder_set_spatialization_behavior: None,
            builder_set_data_callback: None,
            builder_set_error_callback: None,
            builder_set_partial_data_callback: None,
            builder_set_presentation_end_callback: None,
            builder_set_routing_changed_callback: None,
            stream_close: None,
            stream_release: None,
            stream_request_start: None,
            stream_request_pause: None,
            stream_request_flush: None,
            stream_request_stop: None,
            stream_write: None,
            stream_read: None,
            stream_wait_for_state_change: None,
            stream_get_timestamp: None,
            stream_set_buffer_size: None,
            stream_get_channel_count: None,
            stream_get_device_id: None,
            stream_get_format: None,
            stream_get_sample_rate: None,
            stream_get_sharing_mode: None,
            stream_get_performance_mode: None,
            stream_get_buffer_capacity: None,
            stream_get_buffer_size: None,
            stream_get_frames_per_burst: None,
            stream_get_state: None,
            stream_get_xrun_count: None,
            stream_get_frames_read: None,
            stream_get_frames_written: None,
            stream_get_usage: None,
            stream_get_content_type: None,
            stream_get_input_preset: None,
            stream_get_session_id: None,
            stream_is_privacy_sensitive: None,
            stream_get_allowed_capture_policy: None,
            stream_get_channel_mask: None,
            stream_is_content_spatialized: None,
            stream_get_spatialization_behavior: None,
            stream_get_hardware_channel_count: None,
            stream_get_hardware_sample_rate: None,
            stream_get_hardware_format: None,
            stream_set_offload_delay_padding: None,
            stream_get_offload_delay: None,
            stream_get_offload_padding: None,
            stream_set_offload_end_of_stream: None,
            stream_flush_from_frame: None,
            stream_get_playback_parameters: None,
            stream_set_playback_parameters: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustAAudioOutputSettings {
    pub direction: i32,
    pub device_id: i32,
    pub sample_rate: i32,
    pub channel_count: i32,
    pub channel_mask: u32,
    pub format: i32,
    pub sharing_mode: i32,
    pub performance_mode: i32,
    pub buffer_capacity_in_frames: i32,
    pub frames_per_data_callback: i32,
    pub session_id: i32,
    pub usage: i32,
    pub content_type: i32,
    pub allowed_capture_policy: i32,
    pub is_content_spatialized: bool,
    pub spatialization_behavior: i32,
    pub package_name: *const c_char,
    pub attribution_tag: *const c_char,
    pub data_callback: OboeRustAAudioDataCallback,
    pub error_callback: OboeRustAAudioErrorCallback,
    pub partial_data_callback: OboeRustAAudioPartialDataCallback,
    pub presentation_end_callback: OboeRustAAudioPresentationCallback,
    pub routing_changed_callback: OboeRustAAudioRoutingChangedCallback,
    pub user_data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustAAudioOutputProperties {
    pub result: i32,
    pub raw_stream: *mut c_void,
    pub channel_count: i32,
    pub device_id: i32,
    pub sample_rate: i32,
    pub format: i32,
    pub sharing_mode: i32,
    pub performance_mode: i32,
    pub buffer_capacity_in_frames: i32,
    pub buffer_size_in_frames: i32,
    pub frames_per_burst: i32,
    pub usage: i32,
    pub content_type: i32,
    pub input_preset: i32,
    pub session_id: i32,
    pub allowed_capture_policy: i32,
    pub channel_mask: u32,
    pub is_content_spatialized: bool,
    pub spatialization_behavior: i32,
    pub hardware_channel_count: i32,
    pub hardware_sample_rate: i32,
    pub hardware_format: i32,
}

impl Default for OboeRustAAudioOutputProperties {
    fn default() -> Self {
        Self {
            result: RESULT_ERROR_INTERNAL,
            raw_stream: ptr::null_mut(),
            channel_count: 0,
            device_id: 0,
            sample_rate: 0,
            format: 0,
            sharing_mode: 0,
            performance_mode: 0,
            buffer_capacity_in_frames: 0,
            buffer_size_in_frames: 0,
            frames_per_burst: 0,
            usage: 0,
            content_type: 0,
            input_preset: 0,
            session_id: 0,
            allowed_capture_policy: 0,
            channel_mask: 0,
            is_content_spatialized: false,
            spatialization_behavior: 0,
            hardware_channel_count: 0,
            hardware_sample_rate: 0,
            hardware_format: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustAAudioInputSettings {
    pub direction: i32,
    pub device_id: i32,
    pub sample_rate: i32,
    pub channel_count: i32,
    pub channel_mask: u32,
    pub format: i32,
    pub sharing_mode: i32,
    pub performance_mode: i32,
    pub buffer_capacity_in_frames: i32,
    pub frames_per_data_callback: i32,
    pub input_preset: i32,
    pub privacy_sensitive_mode: i32,
    pub privacy_sensitive_mode_unspecified: i32,
    pub privacy_sensitive_mode_enabled: i32,
    pub privacy_sensitive_mode_disabled: i32,
    pub session_id: i32,
    pub usage: i32,
    pub content_type: i32,
    pub allowed_capture_policy: i32,
    pub package_name: *const c_char,
    pub attribution_tag: *const c_char,
    pub is_content_spatialized: bool,
    pub spatialization_behavior: i32,
    pub data_callback: OboeRustAAudioDataCallback,
    pub error_callback: OboeRustAAudioErrorCallback,
    pub partial_data_callback: OboeRustAAudioPartialDataCallback,
    pub presentation_end_callback: OboeRustAAudioPresentationCallback,
    pub routing_changed_callback: OboeRustAAudioRoutingChangedCallback,
    pub user_data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustAAudioInputProperties {
    pub result: i32,
    pub raw_stream: *mut c_void,
    pub channel_count: i32,
    pub device_id: i32,
    pub sample_rate: i32,
    pub format: i32,
    pub sharing_mode: i32,
    pub performance_mode: i32,
    pub buffer_capacity_in_frames: i32,
    pub buffer_size_in_frames: i32,
    pub frames_per_burst: i32,
    pub usage: i32,
    pub content_type: i32,
    pub input_preset: i32,
    pub session_id: i32,
    pub allowed_capture_policy: i32,
    pub privacy_sensitive_mode: i32,
    pub channel_mask: u32,
    pub is_content_spatialized: bool,
    pub spatialization_behavior: i32,
    pub hardware_channel_count: i32,
    pub hardware_sample_rate: i32,
    pub hardware_format: i32,
}

impl Default for OboeRustAAudioInputProperties {
    fn default() -> Self {
        Self {
            result: RESULT_ERROR_INTERNAL,
            raw_stream: ptr::null_mut(),
            channel_count: 0,
            device_id: 0,
            sample_rate: 0,
            format: 0,
            sharing_mode: 0,
            performance_mode: 0,
            buffer_capacity_in_frames: 0,
            buffer_size_in_frames: 0,
            frames_per_burst: 0,
            usage: 0,
            content_type: 0,
            input_preset: 0,
            session_id: 0,
            allowed_capture_policy: 0,
            privacy_sensitive_mode: 0,
            channel_mask: 0,
            is_content_spatialized: false,
            spatialization_behavior: 0,
            hardware_channel_count: 0,
            hardware_sample_rate: 0,
            hardware_format: 0,
        }
    }
}

struct AaudioOutputStream {
    stream: *mut c_void,
    platform: OboeRustAAudioPlatform,
    properties: OboeRustAAudioOutputProperties,
    closed: bool,
}

struct AaudioInputStream {
    stream: *mut c_void,
    platform: OboeRustAAudioPlatform,
    properties: OboeRustAAudioInputProperties,
    closed: bool,
}

unsafe fn allocate_stream(value: AaudioOutputStream) -> *mut OboeRustAAudioOutputStream {
    let raw = unsafe { malloc(mem::size_of::<AaudioOutputStream>()) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    let stream = raw.cast::<AaudioOutputStream>();
    unsafe {
        stream.write(value);
    }
    stream.cast::<OboeRustAAudioOutputStream>()
}

unsafe fn allocate_input_stream(value: AaudioInputStream) -> *mut OboeRustAAudioInputStream {
    let raw = unsafe { malloc(mem::size_of::<AaudioInputStream>()) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    let stream = raw.cast::<AaudioInputStream>();
    unsafe {
        stream.write(value);
    }
    stream.cast::<OboeRustAAudioInputStream>()
}

unsafe fn free_stream(handle: *mut OboeRustAAudioOutputStream) {
    let stream = handle.cast::<AaudioOutputStream>();
    unsafe {
        ptr::drop_in_place(stream);
        free(stream.cast::<c_void>());
    }
}

unsafe fn free_input_stream(handle: *mut OboeRustAAudioInputStream) {
    let stream = handle.cast::<AaudioInputStream>();
    unsafe {
        ptr::drop_in_place(stream);
        free(stream.cast::<c_void>());
    }
}

unsafe fn handle_mut<'a>(
    handle: *mut OboeRustAAudioOutputStream,
) -> Option<&'a mut AaudioOutputStream> {
    if handle.is_null() {
        None
    } else {
        unsafe { Some(&mut *handle.cast::<AaudioOutputStream>()) }
    }
}

unsafe fn input_handle_mut<'a>(
    handle: *mut OboeRustAAudioInputStream,
) -> Option<&'a mut AaudioInputStream> {
    if handle.is_null() {
        None
    } else {
        unsafe { Some(&mut *handle.cast::<AaudioInputStream>()) }
    }
}

unsafe fn call_set_i32(
    setter: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    builder: *mut c_void,
    value: i32,
) {
    if let Some(setter) = setter {
        unsafe {
            setter(builder, value);
        }
    }
}

unsafe fn get_i32(
    getter: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    stream: *mut c_void,
    fallback: i32,
) -> i32 {
    if let Some(getter) = getter {
        unsafe { getter(stream) }
    } else {
        fallback
    }
}

unsafe fn get_i64(
    getter: Option<unsafe extern "C" fn(*mut c_void) -> i64>,
    stream: *mut c_void,
    fallback: i64,
) -> i64 {
    if let Some(getter) = getter {
        unsafe { getter(stream) }
    } else {
        fallback
    }
}

unsafe fn close_output_stream(stream: &mut AaudioOutputStream) -> i32 {
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    let result = if let Some(close) = stream.platform.stream_close {
        unsafe { close(stream.stream) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    };
    if result == RESULT_OK {
        stream.stream = ptr::null_mut();
        stream.closed = true;
        stream.properties.raw_stream = ptr::null_mut();
    }
    result
}

unsafe fn close_input_stream(stream: &mut AaudioInputStream) -> i32 {
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    let result = if let Some(close) = stream.platform.stream_close {
        unsafe { close(stream.stream) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    };
    if result == RESULT_OK {
        stream.stream = ptr::null_mut();
        stream.closed = true;
        stream.properties.raw_stream = ptr::null_mut();
    }
    result
}

unsafe fn close_raw_stream(platform: &OboeRustAAudioPlatform, stream: *mut c_void) {
    if stream.is_null() {
        return;
    }
    if let Some(close) = platform.stream_close {
        let _ = unsafe { close(stream) };
    }
}

unsafe fn refresh_output_properties(
    platform: &OboeRustAAudioPlatform,
    settings: &OboeRustAAudioOutputSettings,
    stream: *mut c_void,
    result: i32,
) -> OboeRustAAudioOutputProperties {
    OboeRustAAudioOutputProperties {
        result,
        raw_stream: stream,
        channel_count: unsafe {
            get_i32(
                platform.stream_get_channel_count,
                stream,
                settings.channel_count,
            )
        },
        device_id: unsafe { get_i32(platform.stream_get_device_id, stream, settings.device_id) },
        sample_rate: unsafe {
            get_i32(
                platform.stream_get_sample_rate,
                stream,
                settings.sample_rate,
            )
        },
        format: unsafe { get_i32(platform.stream_get_format, stream, settings.format) },
        sharing_mode: unsafe {
            get_i32(
                platform.stream_get_sharing_mode,
                stream,
                settings.sharing_mode,
            )
        },
        performance_mode: unsafe {
            get_i32(
                platform.stream_get_performance_mode,
                stream,
                settings.performance_mode,
            )
        },
        buffer_capacity_in_frames: unsafe {
            get_i32(
                platform.stream_get_buffer_capacity,
                stream,
                settings.buffer_capacity_in_frames,
            )
        },
        buffer_size_in_frames: unsafe {
            get_i32(
                platform.stream_get_buffer_size,
                stream,
                settings.buffer_capacity_in_frames,
            )
        },
        frames_per_burst: unsafe { get_i32(platform.stream_get_frames_per_burst, stream, 0) },
        usage: unsafe { get_i32(platform.stream_get_usage, stream, settings.usage) },
        content_type: unsafe {
            get_i32(
                platform.stream_get_content_type,
                stream,
                settings.content_type,
            )
        },
        input_preset: unsafe { get_i32(platform.stream_get_input_preset, stream, 0) },
        session_id: unsafe { get_i32(platform.stream_get_session_id, stream, settings.session_id) },
        allowed_capture_policy: unsafe {
            get_i32(
                platform.stream_get_allowed_capture_policy,
                stream,
                settings.allowed_capture_policy,
            )
        },
        channel_mask: if let Some(getter) = platform.stream_get_channel_mask {
            unsafe { getter(stream) }
        } else {
            settings.channel_mask
        },
        is_content_spatialized: if let Some(getter) = platform.stream_is_content_spatialized {
            unsafe { getter(stream) }
        } else {
            settings.is_content_spatialized
        },
        spatialization_behavior: unsafe {
            get_i32(
                platform.stream_get_spatialization_behavior,
                stream,
                settings.spatialization_behavior,
            )
        },
        hardware_channel_count: unsafe {
            get_i32(platform.stream_get_hardware_channel_count, stream, 0)
        },
        hardware_sample_rate: unsafe {
            get_i32(platform.stream_get_hardware_sample_rate, stream, 0)
        },
        hardware_format: unsafe { get_i32(platform.stream_get_hardware_format, stream, 0) },
    }
}

unsafe fn refresh_input_properties(
    platform: &OboeRustAAudioPlatform,
    settings: &OboeRustAAudioInputSettings,
    stream: *mut c_void,
    result: i32,
) -> OboeRustAAudioInputProperties {
    OboeRustAAudioInputProperties {
        result,
        raw_stream: stream,
        channel_count: unsafe {
            get_i32(
                platform.stream_get_channel_count,
                stream,
                settings.channel_count,
            )
        },
        device_id: unsafe { get_i32(platform.stream_get_device_id, stream, settings.device_id) },
        sample_rate: unsafe {
            get_i32(
                platform.stream_get_sample_rate,
                stream,
                settings.sample_rate,
            )
        },
        format: unsafe { get_i32(platform.stream_get_format, stream, settings.format) },
        sharing_mode: unsafe {
            get_i32(
                platform.stream_get_sharing_mode,
                stream,
                settings.sharing_mode,
            )
        },
        performance_mode: unsafe {
            get_i32(
                platform.stream_get_performance_mode,
                stream,
                settings.performance_mode,
            )
        },
        buffer_capacity_in_frames: unsafe {
            get_i32(
                platform.stream_get_buffer_capacity,
                stream,
                settings.buffer_capacity_in_frames,
            )
        },
        buffer_size_in_frames: unsafe {
            get_i32(
                platform.stream_get_buffer_size,
                stream,
                settings.buffer_capacity_in_frames,
            )
        },
        frames_per_burst: unsafe { get_i32(platform.stream_get_frames_per_burst, stream, 0) },
        usage: unsafe { get_i32(platform.stream_get_usage, stream, settings.usage) },
        content_type: unsafe {
            get_i32(
                platform.stream_get_content_type,
                stream,
                settings.content_type,
            )
        },
        input_preset: unsafe {
            get_i32(
                platform.stream_get_input_preset,
                stream,
                settings.input_preset,
            )
        },
        session_id: unsafe { get_i32(platform.stream_get_session_id, stream, settings.session_id) },
        allowed_capture_policy: settings.allowed_capture_policy,
        privacy_sensitive_mode: if let Some(is_privacy_sensitive) =
            platform.stream_is_privacy_sensitive
        {
            if unsafe { is_privacy_sensitive(stream) } {
                settings.privacy_sensitive_mode_enabled
            } else {
                settings.privacy_sensitive_mode_disabled
            }
        } else {
            settings.privacy_sensitive_mode_unspecified
        },
        channel_mask: if let Some(getter) = platform.stream_get_channel_mask {
            unsafe { getter(stream) }
        } else {
            settings.channel_mask
        },
        is_content_spatialized: if let Some(getter) = platform.stream_is_content_spatialized {
            unsafe { getter(stream) }
        } else {
            settings.is_content_spatialized
        },
        spatialization_behavior: unsafe {
            get_i32(
                platform.stream_get_spatialization_behavior,
                stream,
                settings.spatialization_behavior,
            )
        },
        hardware_channel_count: unsafe {
            get_i32(platform.stream_get_hardware_channel_count, stream, 0)
        },
        hardware_sample_rate: unsafe {
            get_i32(platform.stream_get_hardware_sample_rate, stream, 0)
        },
        hardware_format: unsafe { get_i32(platform.stream_get_hardware_format, stream, 0) },
    }
}

/// # Safety
///
/// `platform`, `settings`, and `properties` must be valid pointers. Function pointers in
/// `platform` must follow the AAudio contracts for the lifetime of the returned handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_open(
    platform: *const OboeRustAAudioPlatform,
    settings: *const OboeRustAAudioOutputSettings,
    properties: *mut OboeRustAAudioOutputProperties,
) -> *mut OboeRustAAudioOutputStream {
    if platform.is_null() || settings.is_null() || properties.is_null() {
        return ptr::null_mut();
    }
    let platform = unsafe { *platform };
    let settings = unsafe { *settings };
    unsafe {
        *properties = OboeRustAAudioOutputProperties {
            result: RESULT_ERROR_INTERNAL,
            ..OboeRustAAudioOutputProperties::default()
        };
    }

    let Some(create_stream_builder) = platform.create_stream_builder else {
        return ptr::null_mut();
    };
    let Some(builder_open_stream) = platform.builder_open_stream else {
        return ptr::null_mut();
    };
    let Some(builder_delete) = platform.builder_delete else {
        return ptr::null_mut();
    };

    let mut builder = ptr::null_mut();
    let create_result = unsafe { create_stream_builder(&mut builder) };
    if create_result != RESULT_OK || builder.is_null() {
        unsafe {
            (*properties).result = create_result;
        }
        return ptr::null_mut();
    }

    unsafe {
        call_set_i32(
            platform.builder_set_buffer_capacity_in_frames,
            builder,
            settings.buffer_capacity_in_frames,
        );
        if settings.channel_mask != 0 {
            if let Some(set_channel_mask) = platform.builder_set_channel_mask {
                set_channel_mask(builder, settings.channel_mask);
            } else {
                call_set_i32(
                    platform.builder_set_channel_count,
                    builder,
                    settings.channel_count,
                );
            }
        } else {
            call_set_i32(
                platform.builder_set_channel_count,
                builder,
                settings.channel_count,
            );
        }
        call_set_i32(platform.builder_set_device_id, builder, settings.device_id);
        call_set_i32(platform.builder_set_direction, builder, settings.direction);
        call_set_i32(platform.builder_set_format, builder, settings.format);
        call_set_i32(
            platform.builder_set_frames_per_data_callback,
            builder,
            settings.frames_per_data_callback,
        );
        call_set_i32(
            platform.builder_set_performance_mode,
            builder,
            settings.performance_mode,
        );
        call_set_i32(
            platform.builder_set_sample_rate,
            builder,
            settings.sample_rate,
        );
        call_set_i32(
            platform.builder_set_sharing_mode,
            builder,
            settings.sharing_mode,
        );
        call_set_i32(platform.builder_set_usage, builder, settings.usage);
        call_set_i32(
            platform.builder_set_content_type,
            builder,
            settings.content_type,
        );
        call_set_i32(
            platform.builder_set_session_id,
            builder,
            settings.session_id,
        );
        call_set_i32(
            platform.builder_set_allowed_capture_policy,
            builder,
            settings.allowed_capture_policy,
        );
        if !settings.package_name.is_null() {
            if let Some(set_package_name) = platform.builder_set_package_name {
                set_package_name(builder, settings.package_name);
            }
        }
        if !settings.attribution_tag.is_null() {
            if let Some(set_attribution_tag) = platform.builder_set_attribution_tag {
                set_attribution_tag(builder, settings.attribution_tag);
            }
        }
        if let Some(set_content_spatialized) = platform.builder_set_is_content_spatialized {
            set_content_spatialized(builder, settings.is_content_spatialized);
        }
        call_set_i32(
            platform.builder_set_spatialization_behavior,
            builder,
            settings.spatialization_behavior,
        );
        if settings.data_callback.is_some() {
            if let Some(set_data_callback) = platform.builder_set_data_callback {
                set_data_callback(builder, settings.data_callback, settings.user_data);
            }
        } else if settings.partial_data_callback.is_some() {
            if let Some(set_partial_data_callback) = platform.builder_set_partial_data_callback {
                set_partial_data_callback(
                    builder,
                    settings.partial_data_callback,
                    settings.user_data,
                );
            }
        }
        if settings.error_callback.is_some() {
            if let Some(set_error_callback) = platform.builder_set_error_callback {
                set_error_callback(builder, settings.error_callback, settings.user_data);
            }
        }
        if settings.presentation_end_callback.is_some() {
            if let Some(set_presentation_end_callback) =
                platform.builder_set_presentation_end_callback
            {
                set_presentation_end_callback(
                    builder,
                    settings.presentation_end_callback,
                    settings.user_data,
                );
            }
        }
        if settings.routing_changed_callback.is_some() {
            if let Some(set_routing_changed_callback) =
                platform.builder_set_routing_changed_callback
            {
                set_routing_changed_callback(
                    builder,
                    settings.routing_changed_callback,
                    settings.user_data,
                );
            }
        }
    }

    let mut raw_stream = ptr::null_mut();
    let open_result = unsafe { builder_open_stream(builder, &mut raw_stream) };
    unsafe {
        builder_delete(builder);
    }
    if open_result != RESULT_OK || raw_stream.is_null() {
        unsafe {
            (*properties).result = open_result;
        }
        return ptr::null_mut();
    }

    let cached_properties =
        unsafe { refresh_output_properties(&platform, &settings, raw_stream, open_result) };
    let handle = unsafe {
        allocate_stream(AaudioOutputStream {
            stream: raw_stream,
            platform,
            properties: cached_properties,
            closed: false,
        })
    };
    if handle.is_null() {
        unsafe {
            close_raw_stream(&platform, raw_stream);
        }
        let mut failed_properties = cached_properties;
        failed_properties.result = RESULT_ERROR_INTERNAL;
        failed_properties.raw_stream = ptr::null_mut();
        unsafe {
            *properties = failed_properties;
        }
        return ptr::null_mut();
    }
    unsafe {
        *properties = cached_properties;
    }
    handle
}

/// # Safety
///
/// `platform`, `settings`, and `properties` must be valid pointers. Function pointers in
/// `platform` must follow the AAudio contracts for the lifetime of the returned handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_open(
    platform: *const OboeRustAAudioPlatform,
    settings: *const OboeRustAAudioInputSettings,
    properties: *mut OboeRustAAudioInputProperties,
) -> *mut OboeRustAAudioInputStream {
    if platform.is_null() || settings.is_null() || properties.is_null() {
        return ptr::null_mut();
    }
    let platform = unsafe { *platform };
    let settings = unsafe { *settings };
    unsafe {
        *properties = OboeRustAAudioInputProperties {
            result: RESULT_ERROR_INTERNAL,
            ..OboeRustAAudioInputProperties::default()
        };
    }

    let Some(create_stream_builder) = platform.create_stream_builder else {
        return ptr::null_mut();
    };
    let Some(builder_open_stream) = platform.builder_open_stream else {
        return ptr::null_mut();
    };
    let Some(builder_delete) = platform.builder_delete else {
        return ptr::null_mut();
    };

    let mut builder = ptr::null_mut();
    let create_result = unsafe { create_stream_builder(&mut builder) };
    if create_result != RESULT_OK || builder.is_null() {
        unsafe {
            (*properties).result = create_result;
        }
        return ptr::null_mut();
    }

    unsafe {
        call_set_i32(
            platform.builder_set_buffer_capacity_in_frames,
            builder,
            settings.buffer_capacity_in_frames,
        );
        if settings.channel_mask != 0 {
            if let Some(set_channel_mask) = platform.builder_set_channel_mask {
                set_channel_mask(builder, settings.channel_mask);
            } else {
                call_set_i32(
                    platform.builder_set_channel_count,
                    builder,
                    settings.channel_count,
                );
            }
        } else {
            call_set_i32(
                platform.builder_set_channel_count,
                builder,
                settings.channel_count,
            );
        }
        call_set_i32(platform.builder_set_device_id, builder, settings.device_id);
        call_set_i32(platform.builder_set_direction, builder, settings.direction);
        call_set_i32(platform.builder_set_format, builder, settings.format);
        call_set_i32(
            platform.builder_set_frames_per_data_callback,
            builder,
            settings.frames_per_data_callback,
        );
        call_set_i32(
            platform.builder_set_input_preset,
            builder,
            settings.input_preset,
        );
        call_set_i32(
            platform.builder_set_performance_mode,
            builder,
            settings.performance_mode,
        );
        call_set_i32(
            platform.builder_set_sample_rate,
            builder,
            settings.sample_rate,
        );
        call_set_i32(
            platform.builder_set_sharing_mode,
            builder,
            settings.sharing_mode,
        );
        call_set_i32(platform.builder_set_usage, builder, settings.usage);
        call_set_i32(
            platform.builder_set_content_type,
            builder,
            settings.content_type,
        );
        call_set_i32(
            platform.builder_set_session_id,
            builder,
            settings.session_id,
        );
        if settings.privacy_sensitive_mode != settings.privacy_sensitive_mode_unspecified {
            if let Some(set_privacy_sensitive) = platform.builder_set_privacy_sensitive {
                set_privacy_sensitive(
                    builder,
                    settings.privacy_sensitive_mode == settings.privacy_sensitive_mode_enabled,
                );
            }
        }
        if !settings.package_name.is_null() {
            if let Some(set_package_name) = platform.builder_set_package_name {
                set_package_name(builder, settings.package_name);
            }
        }
        if !settings.attribution_tag.is_null() {
            if let Some(set_attribution_tag) = platform.builder_set_attribution_tag {
                set_attribution_tag(builder, settings.attribution_tag);
            }
        }
        if let Some(set_content_spatialized) = platform.builder_set_is_content_spatialized {
            set_content_spatialized(builder, settings.is_content_spatialized);
        }
        call_set_i32(
            platform.builder_set_spatialization_behavior,
            builder,
            settings.spatialization_behavior,
        );
        if settings.data_callback.is_some() {
            if let Some(set_data_callback) = platform.builder_set_data_callback {
                set_data_callback(builder, settings.data_callback, settings.user_data);
            }
        } else if settings.partial_data_callback.is_some() {
            if let Some(set_partial_data_callback) = platform.builder_set_partial_data_callback {
                set_partial_data_callback(
                    builder,
                    settings.partial_data_callback,
                    settings.user_data,
                );
            }
        }
        if settings.error_callback.is_some() {
            if let Some(set_error_callback) = platform.builder_set_error_callback {
                set_error_callback(builder, settings.error_callback, settings.user_data);
            }
        }
        if settings.presentation_end_callback.is_some() {
            if let Some(set_presentation_end_callback) =
                platform.builder_set_presentation_end_callback
            {
                set_presentation_end_callback(
                    builder,
                    settings.presentation_end_callback,
                    settings.user_data,
                );
            }
        }
        if settings.routing_changed_callback.is_some() {
            if let Some(set_routing_changed_callback) =
                platform.builder_set_routing_changed_callback
            {
                set_routing_changed_callback(
                    builder,
                    settings.routing_changed_callback,
                    settings.user_data,
                );
            }
        }
    }

    let mut raw_stream = ptr::null_mut();
    let open_result = unsafe { builder_open_stream(builder, &mut raw_stream) };
    unsafe {
        builder_delete(builder);
    }
    if open_result != RESULT_OK || raw_stream.is_null() {
        unsafe {
            (*properties).result = open_result;
        }
        return ptr::null_mut();
    }

    let cached_properties =
        unsafe { refresh_input_properties(&platform, &settings, raw_stream, open_result) };
    let handle = unsafe {
        allocate_input_stream(AaudioInputStream {
            stream: raw_stream,
            platform,
            properties: cached_properties,
            closed: false,
        })
    };
    if handle.is_null() {
        unsafe {
            close_raw_stream(&platform, raw_stream);
        }
        let mut failed_properties = cached_properties;
        failed_properties.result = RESULT_ERROR_INTERNAL;
        failed_properties.raw_stream = ptr::null_mut();
        unsafe {
            *properties = failed_properties;
        }
        return ptr::null_mut();
    }
    unsafe {
        *properties = cached_properties;
    }
    handle
}

/// # Safety
///
/// `handle` must be a pointer returned by `oboe_rust_aaudio_output_open`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_destroy(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    let result = if stream.closed || stream.stream.is_null() {
        RESULT_OK
    } else {
        unsafe { close_output_stream(stream) }
    };
    unsafe {
        free_stream(handle);
    }
    result
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_close(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    unsafe { close_output_stream(stream) }
}

unsafe fn request_no_arg(
    handle: *mut OboeRustAAudioOutputStream,
    request: fn(&OboeRustAAudioPlatform) -> Option<unsafe extern "C" fn(*mut c_void) -> i32>,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(request) = request(&stream.platform) {
        unsafe { request(stream.stream) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_request_start(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    unsafe { request_no_arg(handle, |platform| platform.stream_request_start) }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_request_pause(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    unsafe { request_no_arg(handle, |platform| platform.stream_request_pause) }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_request_flush(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    unsafe { request_no_arg(handle, |platform| platform.stream_request_flush) }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_request_stop(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    unsafe { request_no_arg(handle, |platform| platform.stream_request_stop) }
}

/// # Safety
///
/// `handle` must be valid and `buffer` must be readable for `num_frames`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_write(
    handle: *mut OboeRustAAudioOutputStream,
    buffer: *const c_void,
    num_frames: i32,
    timeout_nanoseconds: i64,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    let result = if let Some(write) = stream.platform.stream_write {
        unsafe { write(stream.stream, buffer, num_frames, timeout_nanoseconds) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    };
    if result >= 0 {
        stream.properties.raw_stream = stream.stream;
        stream.properties.result = RESULT_OK;
    }
    result
}

/// # Safety
///
/// `handle` must be valid and `buffer` must be writable for `num_frames`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_read(
    handle: *mut OboeRustAAudioOutputStream,
    buffer: *mut c_void,
    num_frames: i32,
    timeout_nanoseconds: i64,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(read) = stream.platform.stream_read {
        unsafe { read(stream.stream, buffer, num_frames, timeout_nanoseconds) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be valid. `next_state` may be null or writable for one `i32`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_wait_for_state_change(
    handle: *mut OboeRustAAudioOutputStream,
    current_state: i32,
    next_state: *mut i32,
    timeout_nanoseconds: i64,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        if !next_state.is_null() {
            unsafe {
                *next_state = STREAM_STATE_CLOSED;
            }
        }
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        if !next_state.is_null() {
            unsafe {
                *next_state = STREAM_STATE_CLOSED;
            }
        }
        return RESULT_ERROR_CLOSED;
    }
    if let Some(wait_for_state_change) = stream.platform.stream_wait_for_state_change {
        unsafe {
            wait_for_state_change(
                stream.stream,
                current_state,
                next_state,
                timeout_nanoseconds,
            )
        }
    } else {
        if !next_state.is_null() {
            unsafe {
                *next_state = oboe_rust_aaudio_output_get_state(handle);
            }
        }
        RESULT_OK
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_state(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return STREAM_STATE_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        STREAM_STATE_CLOSED
    } else {
        unsafe {
            get_i32(
                stream.platform.stream_get_state,
                stream.stream,
                STREAM_STATE_UNKNOWN,
            )
        }
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_set_buffer_size(
    handle: *mut OboeRustAAudioOutputStream,
    requested_frames: i32,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    let result = if let Some(set_buffer_size) = stream.platform.stream_set_buffer_size {
        unsafe { set_buffer_size(stream.stream, requested_frames) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    };
    if result > 0 {
        stream.properties.buffer_size_in_frames = result;
    }
    result
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_buffer_size(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return 0;
    };
    if !stream.closed && !stream.stream.is_null() {
        stream.properties.buffer_size_in_frames = unsafe {
            get_i32(
                stream.platform.stream_get_buffer_size,
                stream.stream,
                stream.properties.buffer_size_in_frames,
            )
        };
    }
    stream.properties.buffer_size_in_frames
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_xrun_count(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_NULL;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_NULL;
    }
    unsafe {
        get_i32(
            stream.platform.stream_get_xrun_count,
            stream.stream,
            RESULT_ERROR_UNIMPLEMENTED,
        )
    }
}

/// # Safety
///
/// `handle` must be valid. `frame_position` and `time_nanoseconds` must be writable when non-null.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_timestamp(
    handle: *mut OboeRustAAudioOutputStream,
    clock_id: i32,
    frame_position: *mut i64,
    time_nanoseconds: *mut i64,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(get_timestamp) = stream.platform.stream_get_timestamp {
        unsafe { get_timestamp(stream.stream, clock_id, frame_position, time_nanoseconds) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_frames_read(
    handle: *mut OboeRustAAudioOutputStream,
) -> i64 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return 0;
    };
    if !stream.closed && !stream.stream.is_null() {
        stream.properties.result = RESULT_OK;
        let value = unsafe { get_i64(stream.platform.stream_get_frames_read, stream.stream, 0) };
        return value;
    }
    0
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_frames_written(
    handle: *mut OboeRustAAudioOutputStream,
) -> i64 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return 0;
    };
    if !stream.closed && !stream.stream.is_null() {
        unsafe { get_i64(stream.platform.stream_get_frames_written, stream.stream, 0) }
    } else {
        0
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_raw_stream(
    handle: *mut OboeRustAAudioOutputStream,
) -> *mut c_void {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return ptr::null_mut();
    };
    stream.stream
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_release(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    unsafe { request_no_arg(handle, |platform| platform.stream_release) }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_set_offload_delay_padding(
    handle: *mut OboeRustAAudioOutputStream,
    delay_in_frames: i32,
    padding_in_frames: i32,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(set_offload_delay_padding) = stream.platform.stream_set_offload_delay_padding {
        unsafe { set_offload_delay_padding(stream.stream, delay_in_frames, padding_in_frames) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_offload_delay(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    unsafe {
        get_i32(
            stream.platform.stream_get_offload_delay,
            stream.stream,
            RESULT_ERROR_UNIMPLEMENTED,
        )
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_offload_padding(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    unsafe {
        get_i32(
            stream.platform.stream_get_offload_padding,
            stream.stream,
            RESULT_ERROR_UNIMPLEMENTED,
        )
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio output handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_set_offload_end_of_stream(
    handle: *mut OboeRustAAudioOutputStream,
) -> i32 {
    unsafe { request_no_arg(handle, |platform| platform.stream_set_offload_end_of_stream) }
}

/// # Safety
///
/// `handle` must be valid and `position_in_frames` must be writable.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_flush_from_frame(
    handle: *mut OboeRustAAudioOutputStream,
    accuracy: i32,
    position_in_frames: *mut i64,
) -> i32 {
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(flush_from_frame) = stream.platform.stream_flush_from_frame {
        unsafe { flush_from_frame(stream.stream, accuracy, position_in_frames) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` and `parameters` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_get_playback_parameters(
    handle: *mut OboeRustAAudioOutputStream,
    parameters: *mut OboeRustAAudioPlaybackParameters,
) -> i32 {
    if parameters.is_null() {
        return RESULT_ERROR_NULL;
    }
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(get_playback_parameters) = stream.platform.stream_get_playback_parameters {
        unsafe { get_playback_parameters(stream.stream, parameters) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` and `parameters` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_output_set_playback_parameters(
    handle: *mut OboeRustAAudioOutputStream,
    parameters: *const OboeRustAAudioPlaybackParameters,
) -> i32 {
    if parameters.is_null() {
        return RESULT_ERROR_NULL;
    }
    let Some(stream) = (unsafe { handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(set_playback_parameters) = stream.platform.stream_set_playback_parameters {
        unsafe { set_playback_parameters(stream.stream, parameters) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be a pointer returned by `oboe_rust_aaudio_input_open`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_destroy(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    let result = if stream.closed || stream.stream.is_null() {
        RESULT_OK
    } else {
        unsafe { close_input_stream(stream) }
    };
    unsafe {
        free_input_stream(handle);
    }
    result
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_close(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    unsafe { close_input_stream(stream) }
}

unsafe fn input_request_no_arg(
    handle: *mut OboeRustAAudioInputStream,
    request: fn(&OboeRustAAudioPlatform) -> Option<unsafe extern "C" fn(*mut c_void) -> i32>,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(request) = request(&stream.platform) {
        unsafe { request(stream.stream) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_request_start(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    unsafe { input_request_no_arg(handle, |platform| platform.stream_request_start) }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_request_pause(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    unsafe { input_request_no_arg(handle, |platform| platform.stream_request_pause) }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_request_flush(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    unsafe { input_request_no_arg(handle, |platform| platform.stream_request_flush) }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_request_stop(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    unsafe { input_request_no_arg(handle, |platform| platform.stream_request_stop) }
}

/// # Safety
///
/// `handle` must be valid and `buffer` must be readable for `num_frames`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_write(
    handle: *mut OboeRustAAudioInputStream,
    buffer: *const c_void,
    num_frames: i32,
    timeout_nanoseconds: i64,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(write) = stream.platform.stream_write {
        unsafe { write(stream.stream, buffer, num_frames, timeout_nanoseconds) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be valid and `buffer` must be writable for `num_frames`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_read(
    handle: *mut OboeRustAAudioInputStream,
    buffer: *mut c_void,
    num_frames: i32,
    timeout_nanoseconds: i64,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(read) = stream.platform.stream_read {
        unsafe { read(stream.stream, buffer, num_frames, timeout_nanoseconds) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be valid. `next_state` may be null or writable for one `i32`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_wait_for_state_change(
    handle: *mut OboeRustAAudioInputStream,
    current_state: i32,
    next_state: *mut i32,
    timeout_nanoseconds: i64,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        if !next_state.is_null() {
            unsafe {
                *next_state = STREAM_STATE_CLOSED;
            }
        }
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        if !next_state.is_null() {
            unsafe {
                *next_state = STREAM_STATE_CLOSED;
            }
        }
        return RESULT_ERROR_CLOSED;
    }
    if let Some(wait_for_state_change) = stream.platform.stream_wait_for_state_change {
        unsafe {
            wait_for_state_change(
                stream.stream,
                current_state,
                next_state,
                timeout_nanoseconds,
            )
        }
    } else {
        if !next_state.is_null() {
            unsafe {
                *next_state = oboe_rust_aaudio_input_get_state(handle);
            }
        }
        RESULT_OK
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_get_state(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return STREAM_STATE_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        STREAM_STATE_CLOSED
    } else {
        unsafe {
            get_i32(
                stream.platform.stream_get_state,
                stream.stream,
                STREAM_STATE_UNKNOWN,
            )
        }
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_set_buffer_size(
    handle: *mut OboeRustAAudioInputStream,
    requested_frames: i32,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    let result = if let Some(set_buffer_size) = stream.platform.stream_set_buffer_size {
        unsafe { set_buffer_size(stream.stream, requested_frames) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    };
    if result > 0 {
        stream.properties.buffer_size_in_frames = result;
    }
    result
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_get_buffer_size(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return 0;
    };
    if !stream.closed && !stream.stream.is_null() {
        stream.properties.buffer_size_in_frames = unsafe {
            get_i32(
                stream.platform.stream_get_buffer_size,
                stream.stream,
                stream.properties.buffer_size_in_frames,
            )
        };
    }
    stream.properties.buffer_size_in_frames
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_get_xrun_count(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_NULL;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_NULL;
    }
    unsafe {
        get_i32(
            stream.platform.stream_get_xrun_count,
            stream.stream,
            RESULT_ERROR_UNIMPLEMENTED,
        )
    }
}

/// # Safety
///
/// `handle` must be valid. `frame_position` and `time_nanoseconds` must be writable when non-null.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_get_timestamp(
    handle: *mut OboeRustAAudioInputStream,
    clock_id: i32,
    frame_position: *mut i64,
    time_nanoseconds: *mut i64,
) -> i32 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return RESULT_ERROR_CLOSED;
    };
    if stream.closed || stream.stream.is_null() {
        return RESULT_ERROR_CLOSED;
    }
    if let Some(get_timestamp) = stream.platform.stream_get_timestamp {
        unsafe { get_timestamp(stream.stream, clock_id, frame_position, time_nanoseconds) }
    } else {
        RESULT_ERROR_UNIMPLEMENTED
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_get_frames_read(
    handle: *mut OboeRustAAudioInputStream,
) -> i64 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return 0;
    };
    if !stream.closed && !stream.stream.is_null() {
        unsafe { get_i64(stream.platform.stream_get_frames_read, stream.stream, 0) }
    } else {
        0
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_get_frames_written(
    handle: *mut OboeRustAAudioInputStream,
) -> i64 {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return 0;
    };
    if !stream.closed && !stream.stream.is_null() {
        unsafe { get_i64(stream.platform.stream_get_frames_written, stream.stream, 0) }
    } else {
        0
    }
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_get_raw_stream(
    handle: *mut OboeRustAAudioInputStream,
) -> *mut c_void {
    let Some(stream) = (unsafe { input_handle_mut(handle) }) else {
        return ptr::null_mut();
    };
    stream.stream
}

/// # Safety
///
/// `handle` must be a valid Rust AAudio input handle.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_aaudio_input_release(
    handle: *mut OboeRustAAudioInputStream,
) -> i32 {
    unsafe { input_request_no_arg(handle, |platform| platform.stream_release) }
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_playback_parameters_valid(
    fallback_mode: i32,
    stretch_mode: i32,
    fallback_default: i32,
    fallback_mute: i32,
    fallback_fail: i32,
    stretch_default: i32,
    stretch_voice: i32,
) -> bool {
    (fallback_mode == fallback_default
        || fallback_mode == fallback_mute
        || fallback_mode == fallback_fail)
        && (stretch_mode == stretch_default || stretch_mode == stretch_voice)
}
