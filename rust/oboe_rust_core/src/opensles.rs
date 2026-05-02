use core::ffi::c_void;
use core::mem;
use core::ptr;

const AUDIO_FORMAT_UNSPECIFIED: i32 = 0;

const SL_RESULT_SUCCESS: i32 = 0;
const SL_RESULT_INTERNAL_ERROR: i32 = 9;

const PERFORMANCE_MODE_NONE: i32 = 10;
const PERFORMANCE_MODE_POWER_SAVING: i32 = 11;
const PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;

const INPUT_PRESET_GENERIC: i32 = 1;
const INPUT_PRESET_CAMCORDER: i32 = 5;
const INPUT_PRESET_VOICE_RECOGNITION: i32 = 6;
const INPUT_PRESET_VOICE_COMMUNICATION: i32 = 7;
const INPUT_PRESET_UNPROCESSED: i32 = 9;
const INPUT_PRESET_VOICE_PERFORMANCE: i32 = 10;

const USAGE_MEDIA: i32 = 1;
const USAGE_VOICE_COMMUNICATION: i32 = 2;
const USAGE_VOICE_COMMUNICATION_SIGNALLING: i32 = 3;
const USAGE_ALARM: i32 = 4;
const USAGE_NOTIFICATION: i32 = 5;
const USAGE_NOTIFICATION_RINGTONE: i32 = 6;
const USAGE_NOTIFICATION_EVENT: i32 = 10;
const USAGE_GAME: i32 = 14;

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

pub type OboeRustOpenSLESQueueCallback = Option<extern "C" fn(*mut c_void)>;

#[repr(C)]
pub struct OboeRustOpenSLESOutputBackend {
    _private: [u8; 0],
}

#[repr(C)]
pub struct OboeRustOpenSLESInputBackend {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustOpenSLESPlatform {
    pub engine_open: Option<unsafe extern "C" fn() -> i32>,
    pub engine_close: Option<unsafe extern "C" fn()>,
    pub output_mixer_open: Option<unsafe extern "C" fn() -> i32>,
    pub output_mixer_close: Option<unsafe extern "C" fn()>,
    pub output_create_player: Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> i32>,
    pub input_create_recorder:
        Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *mut c_void) -> i32>,
    pub object_get_android_configuration:
        Option<unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32>,
    pub object_realize: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub object_destroy: Option<unsafe extern "C" fn(*mut c_void)>,
    pub object_get_play: Option<unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32>,
    pub object_get_record: Option<unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32>,
    pub object_get_simple_buffer_queue:
        Option<unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32>,
    pub configuration_set_performance_mode: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub configuration_get_performance_mode:
        Option<unsafe extern "C" fn(*mut c_void, *mut i32) -> i32>,
    pub configuration_set_stream_type: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub configuration_set_recording_preset: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub queue_register_callback: Option<
        unsafe extern "C" fn(*mut c_void, OboeRustOpenSLESQueueCallback, *mut c_void) -> i32,
    >,
    pub queue_enqueue: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, i32) -> i32>,
    pub queue_clear: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub queue_get_depth: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub play_set_state: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub play_get_position_millis: Option<unsafe extern "C" fn(*mut c_void, *mut i32) -> i32>,
    pub record_set_state: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub record_get_position_millis: Option<unsafe extern "C" fn(*mut c_void, *mut i32) -> i32>,
}

impl OboeRustOpenSLESPlatform {
    #[cfg(test)]
    pub const fn empty() -> Self {
        Self {
            engine_open: None,
            engine_close: None,
            output_mixer_open: None,
            output_mixer_close: None,
            output_create_player: None,
            input_create_recorder: None,
            object_get_android_configuration: None,
            object_realize: None,
            object_destroy: None,
            object_get_play: None,
            object_get_record: None,
            object_get_simple_buffer_queue: None,
            configuration_set_performance_mode: None,
            configuration_get_performance_mode: None,
            configuration_set_stream_type: None,
            configuration_set_recording_preset: None,
            queue_register_callback: None,
            queue_enqueue: None,
            queue_clear: None,
            queue_get_depth: None,
            play_set_state: None,
            play_get_position_millis: None,
            record_set_state: None,
            record_get_position_millis: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustOpenSLESCommonSettings {
    pub sdk_version: i32,
    pub android_api_n_mr1: i32,
    pub android_api_o_mr1: i32,
    pub opensl_performance_mode: i32,
    pub opensl_performance_none: i32,
    pub opensl_performance_latency: i32,
    pub opensl_performance_latency_effects: i32,
    pub opensl_performance_power_saving: i32,
    pub oboe_performance_none: i32,
    pub oboe_performance_low_latency: i32,
    pub oboe_performance_power_saving: i32,
    pub queue_callback: OboeRustOpenSLESQueueCallback,
    pub queue_callback_user_data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustOpenSLESOutputSettings {
    pub common: OboeRustOpenSLESCommonSettings,
    pub audio_source: *mut c_void,
    pub opensl_stream_type: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustOpenSLESInputSettings {
    pub common: OboeRustOpenSLESCommonSettings,
    pub audio_source: *mut c_void,
    pub audio_sink: *mut c_void,
    pub opensl_recording_preset: i32,
    pub opensl_recording_preset_voice_recognition: i32,
    pub oboe_input_preset: i32,
    pub oboe_input_preset_voice_recognition: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustOpenSLESOutputProperties {
    pub result: i32,
    pub raw_object: *mut c_void,
    pub raw_play: *mut c_void,
    pub raw_queue: *mut c_void,
    pub resolved_performance_mode: i32,
}

impl Default for OboeRustOpenSLESOutputProperties {
    fn default() -> Self {
        Self {
            result: SL_RESULT_INTERNAL_ERROR,
            raw_object: ptr::null_mut(),
            raw_play: ptr::null_mut(),
            raw_queue: ptr::null_mut(),
            resolved_performance_mode: PERFORMANCE_MODE_NONE,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OboeRustOpenSLESInputProperties {
    pub result: i32,
    pub raw_object: *mut c_void,
    pub raw_record: *mut c_void,
    pub raw_queue: *mut c_void,
    pub resolved_performance_mode: i32,
    pub resolved_input_preset: i32,
}

impl Default for OboeRustOpenSLESInputProperties {
    fn default() -> Self {
        Self {
            result: SL_RESULT_INTERNAL_ERROR,
            raw_object: ptr::null_mut(),
            raw_record: ptr::null_mut(),
            raw_queue: ptr::null_mut(),
            resolved_performance_mode: PERFORMANCE_MODE_NONE,
            resolved_input_preset: 0,
        }
    }
}

struct OpenSLESOutputBackend {
    object: *mut c_void,
    play: *mut c_void,
    queue: *mut c_void,
    platform: OboeRustOpenSLESPlatform,
    output_mixer_opened: bool,
    engine_opened: bool,
    closed: bool,
}

struct OpenSLESInputBackend {
    object: *mut c_void,
    record: *mut c_void,
    queue: *mut c_void,
    platform: OboeRustOpenSLESPlatform,
    engine_opened: bool,
    closed: bool,
}

unsafe fn allocate_output_backend(
    value: OpenSLESOutputBackend,
) -> *mut OboeRustOpenSLESOutputBackend {
    let raw = unsafe { malloc(mem::size_of::<OpenSLESOutputBackend>()) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    let backend = raw.cast::<OpenSLESOutputBackend>();
    unsafe {
        backend.write(value);
    }
    backend.cast::<OboeRustOpenSLESOutputBackend>()
}

unsafe fn allocate_input_backend(value: OpenSLESInputBackend) -> *mut OboeRustOpenSLESInputBackend {
    let raw = unsafe { malloc(mem::size_of::<OpenSLESInputBackend>()) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    let backend = raw.cast::<OpenSLESInputBackend>();
    unsafe {
        backend.write(value);
    }
    backend.cast::<OboeRustOpenSLESInputBackend>()
}

unsafe fn free_output_backend(handle: *mut OboeRustOpenSLESOutputBackend) {
    let backend = handle.cast::<OpenSLESOutputBackend>();
    unsafe {
        ptr::drop_in_place(backend);
        free(backend.cast::<c_void>());
    }
}

unsafe fn free_input_backend(handle: *mut OboeRustOpenSLESInputBackend) {
    let backend = handle.cast::<OpenSLESInputBackend>();
    unsafe {
        ptr::drop_in_place(backend);
        free(backend.cast::<c_void>());
    }
}

unsafe fn output_backend_mut<'a>(
    handle: *mut OboeRustOpenSLESOutputBackend,
) -> Option<&'a mut OpenSLESOutputBackend> {
    if handle.is_null() {
        None
    } else {
        unsafe { Some(&mut *handle.cast::<OpenSLESOutputBackend>()) }
    }
}

unsafe fn input_backend_mut<'a>(
    handle: *mut OboeRustOpenSLESInputBackend,
) -> Option<&'a mut OpenSLESInputBackend> {
    if handle.is_null() {
        None
    } else {
        unsafe { Some(&mut *handle.cast::<OpenSLESInputBackend>()) }
    }
}

fn convert_opensl_performance_mode_from_settings(
    opensl_mode: i32,
    settings: &OboeRustOpenSLESCommonSettings,
) -> i32 {
    oboe_rust_opensles_convert_opensl_performance_mode(
        opensl_mode,
        settings.opensl_performance_none,
        settings.opensl_performance_latency,
        settings.opensl_performance_latency_effects,
        settings.opensl_performance_power_saving,
        settings.oboe_performance_none,
        settings.oboe_performance_low_latency,
        settings.oboe_performance_power_saving,
    )
}

unsafe fn configure_performance_mode(
    platform: &OboeRustOpenSLESPlatform,
    settings: &OboeRustOpenSLESCommonSettings,
    config: *mut c_void,
) -> i32 {
    if config.is_null() || settings.sdk_version < settings.android_api_n_mr1 {
        return SL_RESULT_SUCCESS;
    }
    if let Some(set_performance_mode) = platform.configuration_set_performance_mode {
        unsafe { set_performance_mode(config, settings.opensl_performance_mode) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

unsafe fn resolve_performance_mode(
    platform: &OboeRustOpenSLESPlatform,
    settings: &OboeRustOpenSLESCommonSettings,
    config: *mut c_void,
) -> i32 {
    if config.is_null() || settings.sdk_version < settings.android_api_n_mr1 {
        return settings.oboe_performance_none;
    }
    let Some(get_performance_mode) = platform.configuration_get_performance_mode else {
        return settings.oboe_performance_none;
    };
    let mut opensl_mode = settings.opensl_performance_none;
    let mut result = unsafe { get_performance_mode(config, &mut opensl_mode) };
    if settings.sdk_version <= settings.android_api_o_mr1 {
        result = SL_RESULT_SUCCESS;
    }
    if result == SL_RESULT_SUCCESS {
        convert_opensl_performance_mode_from_settings(opensl_mode, settings)
    } else {
        settings.oboe_performance_none
    }
}

unsafe fn get_configuration(
    platform: &OboeRustOpenSLESPlatform,
    object: *mut c_void,
) -> *mut c_void {
    let Some(get_configuration) = platform.object_get_android_configuration else {
        return ptr::null_mut();
    };
    let mut config = ptr::null_mut();
    let result = unsafe { get_configuration(object, &mut config) };
    if result == SL_RESULT_SUCCESS {
        config
    } else {
        ptr::null_mut()
    }
}

unsafe fn cleanup_output_open(
    platform: &OboeRustOpenSLESPlatform,
    object: *mut c_void,
    output_mixer_opened: bool,
    engine_opened: bool,
) {
    if !object.is_null() {
        if let Some(destroy) = platform.object_destroy {
            unsafe {
                destroy(object);
            }
        }
    }
    if output_mixer_opened {
        if let Some(close) = platform.output_mixer_close {
            unsafe {
                close();
            }
        }
    }
    if engine_opened {
        if let Some(close) = platform.engine_close {
            unsafe {
                close();
            }
        }
    }
}

unsafe fn cleanup_input_open(
    platform: &OboeRustOpenSLESPlatform,
    object: *mut c_void,
    engine_opened: bool,
) {
    if !object.is_null() {
        if let Some(destroy) = platform.object_destroy {
            unsafe {
                destroy(object);
            }
        }
    }
    if engine_opened {
        if let Some(close) = platform.engine_close {
            unsafe {
                close();
            }
        }
    }
}

unsafe fn close_output_backend(backend: &mut OpenSLESOutputBackend) -> i32 {
    if backend.closed {
        return SL_RESULT_SUCCESS;
    }
    if !backend.object.is_null() {
        if let Some(destroy) = backend.platform.object_destroy {
            unsafe {
                destroy(backend.object);
            }
        } else {
            return SL_RESULT_INTERNAL_ERROR;
        }
    }
    if backend.output_mixer_opened {
        if let Some(close) = backend.platform.output_mixer_close {
            unsafe {
                close();
            }
        }
    }
    if backend.engine_opened {
        if let Some(close) = backend.platform.engine_close {
            unsafe {
                close();
            }
        }
    }
    backend.object = ptr::null_mut();
    backend.play = ptr::null_mut();
    backend.queue = ptr::null_mut();
    backend.output_mixer_opened = false;
    backend.engine_opened = false;
    backend.closed = true;
    SL_RESULT_SUCCESS
}

unsafe fn close_input_backend(backend: &mut OpenSLESInputBackend) -> i32 {
    if backend.closed {
        return SL_RESULT_SUCCESS;
    }
    if !backend.object.is_null() {
        if let Some(destroy) = backend.platform.object_destroy {
            unsafe {
                destroy(backend.object);
            }
        } else {
            return SL_RESULT_INTERNAL_ERROR;
        }
    }
    if backend.engine_opened {
        if let Some(close) = backend.platform.engine_close {
            unsafe {
                close();
            }
        }
    }
    backend.object = ptr::null_mut();
    backend.record = ptr::null_mut();
    backend.queue = ptr::null_mut();
    backend.engine_opened = false;
    backend.closed = true;
    SL_RESULT_SUCCESS
}

unsafe fn output_queue_request(
    handle: *mut OboeRustOpenSLESOutputBackend,
    request: fn(&OboeRustOpenSLESPlatform) -> Option<unsafe extern "C" fn(*mut c_void) -> i32>,
) -> i32 {
    let Some(backend) = (unsafe { output_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    if backend.closed || backend.queue.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    if let Some(request) = request(&backend.platform) {
        unsafe { request(backend.queue) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

/// # Safety
///
/// `platform`, `settings`, and `properties` must be valid pointers for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_output_open(
    platform: *const OboeRustOpenSLESPlatform,
    settings: *const OboeRustOpenSLESOutputSettings,
    properties: *mut OboeRustOpenSLESOutputProperties,
) -> *mut OboeRustOpenSLESOutputBackend {
    if platform.is_null() || settings.is_null() || properties.is_null() {
        return ptr::null_mut();
    }
    let platform = unsafe { *platform };
    let settings = unsafe { *settings };
    let mut props = OboeRustOpenSLESOutputProperties::default();
    let mut output_mixer_opened = false;
    let mut object = ptr::null_mut();
    let mut play = ptr::null_mut();
    let mut queue = ptr::null_mut();

    let Some(engine_open) = platform.engine_open else {
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    let mut result = unsafe { engine_open() };
    if result != SL_RESULT_SUCCESS {
        props.result = result;
        unsafe { *properties = props };
        return ptr::null_mut();
    }
    let engine_opened = true;

    let Some(output_mixer_open) = platform.output_mixer_open else {
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { output_mixer_open() };
    if result != SL_RESULT_SUCCESS {
        props.result = result;
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }
    output_mixer_opened = true;

    let Some(create_player) = platform.output_create_player else {
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { create_player(&mut object, settings.audio_source) };
    if result != SL_RESULT_SUCCESS || object.is_null() {
        props.result = result;
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let config = unsafe { get_configuration(&platform, object) };
    if !config.is_null() {
        result = unsafe { configure_performance_mode(&platform, &settings.common, config) };
        if result != SL_RESULT_SUCCESS {
            props.result = result;
            unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
            unsafe { *properties = props };
            return ptr::null_mut();
        }
        if let Some(set_stream_type) = platform.configuration_set_stream_type {
            result = unsafe { set_stream_type(config, settings.opensl_stream_type) };
            if result != SL_RESULT_SUCCESS {
                props.result = result;
                unsafe {
                    cleanup_output_open(&platform, object, output_mixer_opened, engine_opened)
                };
                unsafe { *properties = props };
                return ptr::null_mut();
            }
        }
    }

    let Some(realize) = platform.object_realize else {
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { realize(object) };
    if result != SL_RESULT_SUCCESS {
        props.result = result;
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let Some(get_play) = platform.object_get_play else {
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { get_play(object, &mut play) };
    if result != SL_RESULT_SUCCESS || play.is_null() {
        props.result = result;
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let Some(get_queue) = platform.object_get_simple_buffer_queue else {
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { get_queue(object, &mut queue) };
    if result != SL_RESULT_SUCCESS || queue.is_null() {
        props.result = result;
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let Some(register_callback) = platform.queue_register_callback else {
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe {
        register_callback(
            queue,
            settings.common.queue_callback,
            settings.common.queue_callback_user_data,
        )
    };
    if result != SL_RESULT_SUCCESS {
        props.result = result;
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    props.result = SL_RESULT_SUCCESS;
    props.raw_object = object;
    props.raw_play = play;
    props.raw_queue = queue;
    props.resolved_performance_mode =
        unsafe { resolve_performance_mode(&platform, &settings.common, config) };

    let handle = unsafe {
        allocate_output_backend(OpenSLESOutputBackend {
            object,
            play,
            queue,
            platform,
            output_mixer_opened,
            engine_opened,
            closed: false,
        })
    };
    if handle.is_null() {
        props.result = SL_RESULT_INTERNAL_ERROR;
        unsafe { cleanup_output_open(&platform, object, output_mixer_opened, engine_opened) };
    }
    unsafe {
        *properties = props;
    }
    handle
}

/// # Safety
///
/// `platform`, `settings`, and `properties` must be valid pointers for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_input_open(
    platform: *const OboeRustOpenSLESPlatform,
    settings: *const OboeRustOpenSLESInputSettings,
    properties: *mut OboeRustOpenSLESInputProperties,
) -> *mut OboeRustOpenSLESInputBackend {
    if platform.is_null() || settings.is_null() || properties.is_null() {
        return ptr::null_mut();
    }
    let platform = unsafe { *platform };
    let settings = unsafe { *settings };
    let mut props = OboeRustOpenSLESInputProperties {
        resolved_input_preset: settings.oboe_input_preset,
        ..OboeRustOpenSLESInputProperties::default()
    };
    let mut object = ptr::null_mut();
    let mut record = ptr::null_mut();
    let mut queue = ptr::null_mut();

    let Some(engine_open) = platform.engine_open else {
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    let mut result = unsafe { engine_open() };
    if result != SL_RESULT_SUCCESS {
        props.result = result;
        unsafe { *properties = props };
        return ptr::null_mut();
    }
    let engine_opened = true;

    let Some(create_recorder) = platform.input_create_recorder else {
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { create_recorder(&mut object, settings.audio_source, settings.audio_sink) };
    if result != SL_RESULT_SUCCESS || object.is_null() {
        props.result = result;
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let config = unsafe { get_configuration(&platform, object) };
    if !config.is_null() {
        if let Some(set_recording_preset) = platform.configuration_set_recording_preset {
            result = unsafe { set_recording_preset(config, settings.opensl_recording_preset) };
            if result != SL_RESULT_SUCCESS
                && settings.opensl_recording_preset
                    != settings.opensl_recording_preset_voice_recognition
            {
                let _ = unsafe {
                    set_recording_preset(config, settings.opensl_recording_preset_voice_recognition)
                };
                props.resolved_input_preset = settings.oboe_input_preset_voice_recognition;
            }
        }
        result = unsafe { configure_performance_mode(&platform, &settings.common, config) };
        if result != SL_RESULT_SUCCESS {
            props.result = result;
            unsafe { cleanup_input_open(&platform, object, engine_opened) };
            unsafe { *properties = props };
            return ptr::null_mut();
        }
    }

    let Some(realize) = platform.object_realize else {
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { realize(object) };
    if result != SL_RESULT_SUCCESS {
        props.result = result;
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let Some(get_record) = platform.object_get_record else {
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { get_record(object, &mut record) };
    if result != SL_RESULT_SUCCESS || record.is_null() {
        props.result = result;
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let Some(get_queue) = platform.object_get_simple_buffer_queue else {
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe { get_queue(object, &mut queue) };
    if result != SL_RESULT_SUCCESS || queue.is_null() {
        props.result = result;
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    let Some(register_callback) = platform.queue_register_callback else {
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    };
    result = unsafe {
        register_callback(
            queue,
            settings.common.queue_callback,
            settings.common.queue_callback_user_data,
        )
    };
    if result != SL_RESULT_SUCCESS {
        props.result = result;
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
        unsafe { *properties = props };
        return ptr::null_mut();
    }

    props.result = SL_RESULT_SUCCESS;
    props.raw_object = object;
    props.raw_record = record;
    props.raw_queue = queue;
    props.resolved_performance_mode =
        unsafe { resolve_performance_mode(&platform, &settings.common, config) };

    let handle = unsafe {
        allocate_input_backend(OpenSLESInputBackend {
            object,
            record,
            queue,
            platform,
            engine_opened,
            closed: false,
        })
    };
    if handle.is_null() {
        props.result = SL_RESULT_INTERNAL_ERROR;
        unsafe { cleanup_input_open(&platform, object, engine_opened) };
    }
    unsafe {
        *properties = props;
    }
    handle
}

/// # Safety
///
/// `handle` must be a pointer returned by `oboe_rust_opensles_output_open`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_output_destroy(
    handle: *mut OboeRustOpenSLESOutputBackend,
) -> i32 {
    let Some(backend) = (unsafe { output_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    let result = unsafe { close_output_backend(backend) };
    unsafe {
        free_output_backend(handle);
    }
    result
}

/// # Safety
///
/// `handle` must be a pointer returned by `oboe_rust_opensles_input_open`.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_input_destroy(
    handle: *mut OboeRustOpenSLESInputBackend,
) -> i32 {
    let Some(backend) = (unsafe { input_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    let result = unsafe { close_input_backend(backend) };
    unsafe {
        free_input_backend(handle);
    }
    result
}

/// # Safety
///
/// `handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_output_set_play_state(
    handle: *mut OboeRustOpenSLESOutputBackend,
    state: i32,
) -> i32 {
    let Some(backend) = (unsafe { output_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    if backend.closed || backend.play.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    if let Some(set_state) = backend.platform.play_set_state {
        unsafe { set_state(backend.play, state) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

/// # Safety
///
/// `handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_input_set_record_state(
    handle: *mut OboeRustOpenSLESInputBackend,
    state: i32,
) -> i32 {
    let Some(backend) = (unsafe { input_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    if backend.closed || backend.record.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    if let Some(set_state) = backend.platform.record_set_state {
        unsafe { set_state(backend.record, state) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

/// # Safety
///
/// `handle` and `buffer` must be valid for `num_bytes` bytes.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_output_enqueue(
    handle: *mut OboeRustOpenSLESOutputBackend,
    buffer: *mut c_void,
    num_bytes: i32,
) -> i32 {
    let Some(backend) = (unsafe { output_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    if backend.closed || backend.queue.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    if let Some(enqueue) = backend.platform.queue_enqueue {
        unsafe { enqueue(backend.queue, buffer, num_bytes) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

/// # Safety
///
/// `handle` and `buffer` must be valid for `num_bytes` bytes.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_input_enqueue(
    handle: *mut OboeRustOpenSLESInputBackend,
    buffer: *mut c_void,
    num_bytes: i32,
) -> i32 {
    let Some(backend) = (unsafe { input_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    if backend.closed || backend.queue.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    if let Some(enqueue) = backend.platform.queue_enqueue {
        unsafe { enqueue(backend.queue, buffer, num_bytes) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

/// # Safety
///
/// `handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_output_clear_queue(
    handle: *mut OboeRustOpenSLESOutputBackend,
) -> i32 {
    unsafe { output_queue_request(handle, |platform| platform.queue_clear) }
}

/// # Safety
///
/// `handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_output_get_buffer_depth(
    handle: *mut OboeRustOpenSLESOutputBackend,
) -> i32 {
    let Some(backend) = (unsafe { output_backend_mut(handle) }) else {
        return -1;
    };
    if backend.closed || backend.queue.is_null() {
        return -1;
    }
    if let Some(get_depth) = backend.platform.queue_get_depth {
        unsafe { get_depth(backend.queue) }
    } else {
        -1
    }
}

/// # Safety
///
/// `handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_input_get_buffer_depth(
    handle: *mut OboeRustOpenSLESInputBackend,
) -> i32 {
    let Some(backend) = (unsafe { input_backend_mut(handle) }) else {
        return -1;
    };
    if backend.closed || backend.queue.is_null() {
        return -1;
    }
    if let Some(get_depth) = backend.platform.queue_get_depth {
        unsafe { get_depth(backend.queue) }
    } else {
        -1
    }
}

/// # Safety
///
/// `handle` and `position_millis` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_output_get_position_millis(
    handle: *mut OboeRustOpenSLESOutputBackend,
    position_millis: *mut i32,
) -> i32 {
    if position_millis.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    let Some(backend) = (unsafe { output_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    if backend.closed || backend.play.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    if let Some(get_position) = backend.platform.play_get_position_millis {
        unsafe { get_position(backend.play, position_millis) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

/// # Safety
///
/// `handle` and `position_millis` must be valid.
#[no_mangle]
pub unsafe extern "C" fn oboe_rust_opensles_input_get_position_millis(
    handle: *mut OboeRustOpenSLESInputBackend,
    position_millis: *mut i32,
) -> i32 {
    if position_millis.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    let Some(backend) = (unsafe { input_backend_mut(handle) }) else {
        return SL_RESULT_INTERNAL_ERROR;
    };
    if backend.closed || backend.record.is_null() {
        return SL_RESULT_INTERNAL_ERROR;
    }
    if let Some(get_position) = backend.platform.record_get_position_millis {
        unsafe { get_position(backend.record, position_millis) }
    } else {
        SL_RESULT_INTERNAL_ERROR
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_round_up_divide(x: i32, n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    (x + n - 1) / n
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_channel_mask_default(
    channel_count: i32,
    sdk_version: i32,
    android_api_n: i32,
    channel_count_max: i32,
    unknown_channel_mask: i32,
    non_positional_mask: i32,
) -> i32 {
    if channel_count < 0 || channel_count > channel_count_max {
        return unknown_channel_mask;
    }
    let bitfield = (1u32 << channel_count as u32).wrapping_sub(1) as i32;
    if sdk_version >= android_api_n {
        bitfield | non_positional_mask
    } else {
        bitfield
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_input_channel_mask(
    channel_count: i32,
    default_mask: i32,
    _front_center: i32,
    front_left: i32,
    front_right: i32,
) -> i32 {
    match channel_count {
        1 => front_left,
        2 => front_left | front_right,
        _ => default_mask,
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_output_channel_mask(
    channel_count: i32,
    default_mask: i32,
    front_center: i32,
    stereo: i32,
    quad: i32,
    five_dot_one: i32,
    seven_dot_one: i32,
) -> i32 {
    match channel_count {
        1 => front_center,
        2 => stereo,
        4 => quad,
        6 => five_dot_one,
        8 => seven_dot_one,
        _ => default_mask,
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_optimal_buffer_queue_length(
    default_queue_length: i32,
    max_queue_length: i32,
    buffer_capacity_in_frames: i32,
    double_buffer_count: i32,
    frames_per_callback: i32,
    likely_frames_per_burst: i32,
) -> i32 {
    let mut queue_length = default_queue_length;
    let mut min_capacity = buffer_capacity_in_frames;
    min_capacity = min_capacity.max(double_buffer_count * frames_per_callback);
    if min_capacity > 0 {
        let from_capacity =
            oboe_rust_opensles_round_up_divide(min_capacity, likely_frames_per_burst);
        queue_length = queue_length.max(from_capacity);
    }
    queue_length.min(max_queue_length)
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_estimate_native_frames_per_burst(
    default_frames_per_burst: i32,
    default_sample_rate: i32,
    stream_sample_rate: i32,
    performance_mode: i32,
    sdk_version: i32,
    android_api_n_mr1: i32,
    performance_mode_low_latency: i32,
    high_latency_buffer_size_millis: i32,
    millis_per_second: i32,
) -> i32 {
    let mut frames_per_burst = default_frames_per_burst.max(16);
    let mut sample_rate = 48_000;
    if default_sample_rate > 0 {
        sample_rate = default_sample_rate;
    }
    if stream_sample_rate > 0 {
        sample_rate = stream_sample_rate;
    }
    let high_latency_frames = high_latency_buffer_size_millis * sample_rate / millis_per_second;
    if sdk_version >= android_api_n_mr1
        && performance_mode != performance_mode_low_latency
        && frames_per_burst < high_latency_frames
    {
        let bursts = oboe_rust_opensles_round_up_divide(high_latency_frames, frames_per_burst);
        frames_per_burst *= bursts;
    }
    frames_per_burst
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_configured_callback_frames(
    frames_per_callback: i32,
    frames_per_burst: i32,
) -> i32 {
    if frames_per_callback > 0 {
        frames_per_callback
    } else {
        frames_per_burst
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_select_default_format(
    format: i32,
    sdk_version: i32,
    minimum_float_api: i32,
    i16_format: i32,
    float_format: i32,
) -> i32 {
    if format == AUDIO_FORMAT_UNSPECIFIED {
        if sdk_version < minimum_float_api {
            i16_format
        } else {
            float_format
        }
    } else {
        format
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_convert_oboe_performance_mode(
    oboe_mode: i32,
    session_id: i32,
    session_id_none: i32,
    opensl_none: i32,
    opensl_latency: i32,
    opensl_latency_effects: i32,
    opensl_power_saving: i32,
) -> i32 {
    match oboe_mode {
        PERFORMANCE_MODE_LOW_LATENCY => {
            if session_id == session_id_none {
                opensl_latency
            } else {
                opensl_latency_effects
            }
        }
        PERFORMANCE_MODE_POWER_SAVING => opensl_power_saving,
        PERFORMANCE_MODE_NONE => opensl_none,
        _ => opensl_none,
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_convert_opensl_performance_mode(
    opensl_mode: i32,
    _opensl_none: i32,
    opensl_latency: i32,
    opensl_latency_effects: i32,
    opensl_power_saving: i32,
    oboe_none: i32,
    oboe_low_latency: i32,
    oboe_power_saving: i32,
) -> i32 {
    if opensl_mode == opensl_latency || opensl_mode == opensl_latency_effects {
        oboe_low_latency
    } else if opensl_mode == opensl_power_saving {
        oboe_power_saving
    } else {
        oboe_none
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_normalize_input_preset(
    input_preset: i32,
    voice_performance: i32,
    voice_recognition: i32,
) -> i32 {
    if input_preset == voice_performance {
        voice_recognition
    } else {
        input_preset
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_convert_input_preset(
    input_preset: i32,
    opensl_none: i32,
    opensl_generic: i32,
    opensl_camcorder: i32,
    opensl_voice_recognition: i32,
    opensl_voice_communication: i32,
    opensl_unprocessed: i32,
) -> i32 {
    match input_preset {
        INPUT_PRESET_GENERIC => opensl_generic,
        INPUT_PRESET_CAMCORDER => opensl_camcorder,
        INPUT_PRESET_VOICE_RECOGNITION | INPUT_PRESET_VOICE_PERFORMANCE => opensl_voice_recognition,
        INPUT_PRESET_VOICE_COMMUNICATION => opensl_voice_communication,
        INPUT_PRESET_UNPROCESSED => opensl_unprocessed,
        _ => opensl_none,
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_convert_output_usage(
    usage: i32,
    opensl_media: i32,
    opensl_voice: i32,
    opensl_alarm: i32,
    opensl_notification: i32,
    opensl_ring: i32,
    opensl_system: i32,
) -> i32 {
    match usage {
        USAGE_MEDIA | USAGE_GAME => opensl_media,
        USAGE_VOICE_COMMUNICATION | USAGE_VOICE_COMMUNICATION_SIGNALLING => opensl_voice,
        USAGE_ALARM => opensl_alarm,
        USAGE_NOTIFICATION | USAGE_NOTIFICATION_EVENT => opensl_notification,
        USAGE_NOTIFICATION_RINGTONE => opensl_ring,
        _ => opensl_system,
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_opensles_output_position_millis(
    frames_read: i64,
    sample_rate: i32,
    millis_per_second: i32,
) -> i64 {
    if sample_rate <= 0 {
        return 0;
    }
    frames_read * millis_per_second as i64 / sample_rate as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
    use std::sync::Mutex;

    const OBJECT_PTR: *mut c_void = 0x1000 as *mut c_void;
    const CONFIG_PTR: *mut c_void = 0x2000 as *mut c_void;
    const PLAY_PTR: *mut c_void = 0x3000 as *mut c_void;
    const RECORD_PTR: *mut c_void = 0x4000 as *mut c_void;
    const QUEUE_PTR: *mut c_void = 0x5000 as *mut c_void;
    const AUDIO_SOURCE_PTR: *mut c_void = 0x6000 as *mut c_void;
    const AUDIO_SINK_PTR: *mut c_void = 0x7000 as *mut c_void;
    const USER_DATA_PTR: *mut c_void = 0x8000 as *mut c_void;

    const SDK_P: i32 = 28;
    const API_N_MR1: i32 = 25;
    const API_O_MR1: i32 = 27;
    const OPENSL_PERFORMANCE_NONE: i32 = 0;
    const OPENSL_PERFORMANCE_LATENCY: i32 = 1;
    const OPENSL_PERFORMANCE_LATENCY_EFFECTS: i32 = 2;
    const OPENSL_PERFORMANCE_POWER_SAVING: i32 = 3;
    const OPENSL_STREAM_MEDIA: i32 = 4;
    const OPENSL_RECORDING_PRESET_GENERIC: i32 = 5;
    const OPENSL_RECORDING_PRESET_VOICE_RECOGNITION: i32 = 6;
    const INPUT_PRESET_GENERIC_VALUE: i32 = 1;
    const INPUT_PRESET_VOICE_RECOGNITION_VALUE: i32 = 6;
    const PLAYING_STATE: i32 = 3;
    const RECORDING_STATE: i32 = 3;

    static ENGINE_OPENED: AtomicI32 = AtomicI32::new(0);
    static ENGINE_CLOSED: AtomicI32 = AtomicI32::new(0);
    static MIXER_OPENED: AtomicI32 = AtomicI32::new(0);
    static MIXER_CLOSED: AtomicI32 = AtomicI32::new(0);
    static PLAYER_CREATED: AtomicI32 = AtomicI32::new(0);
    static RECORDER_CREATED: AtomicI32 = AtomicI32::new(0);
    static OBJECT_REALIZED: AtomicI32 = AtomicI32::new(0);
    static OBJECT_DESTROYED: AtomicI32 = AtomicI32::new(0);
    static CALLBACK_REGISTERED: AtomicI32 = AtomicI32::new(0);
    static QUEUE_ENQUEUED: AtomicI32 = AtomicI32::new(0);
    static QUEUE_CLEARED: AtomicI32 = AtomicI32::new(0);
    static PLAY_STATE_SET: AtomicI32 = AtomicI32::new(0);
    static RECORD_STATE_SET: AtomicI32 = AtomicI32::new(0);
    static STREAM_TYPE_SET: AtomicI32 = AtomicI32::new(0);
    static RECORDING_PRESET_SET: AtomicI32 = AtomicI32::new(0);
    static FALLBACK_RECORDING_PRESET_SET: AtomicI32 = AtomicI32::new(0);
    static PERFORMANCE_MODE_SET: AtomicI32 = AtomicI32::new(0);
    static CALLBACK_USER_DATA_MATCHED: AtomicBool = AtomicBool::new(false);
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset() {
        ENGINE_OPENED.store(0, Ordering::SeqCst);
        ENGINE_CLOSED.store(0, Ordering::SeqCst);
        MIXER_OPENED.store(0, Ordering::SeqCst);
        MIXER_CLOSED.store(0, Ordering::SeqCst);
        PLAYER_CREATED.store(0, Ordering::SeqCst);
        RECORDER_CREATED.store(0, Ordering::SeqCst);
        OBJECT_REALIZED.store(0, Ordering::SeqCst);
        OBJECT_DESTROYED.store(0, Ordering::SeqCst);
        CALLBACK_REGISTERED.store(0, Ordering::SeqCst);
        QUEUE_ENQUEUED.store(0, Ordering::SeqCst);
        QUEUE_CLEARED.store(0, Ordering::SeqCst);
        PLAY_STATE_SET.store(0, Ordering::SeqCst);
        RECORD_STATE_SET.store(0, Ordering::SeqCst);
        STREAM_TYPE_SET.store(0, Ordering::SeqCst);
        RECORDING_PRESET_SET.store(0, Ordering::SeqCst);
        FALLBACK_RECORDING_PRESET_SET.store(0, Ordering::SeqCst);
        PERFORMANCE_MODE_SET.store(0, Ordering::SeqCst);
        CALLBACK_USER_DATA_MATCHED.store(false, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_engine_open() -> i32 {
        ENGINE_OPENED.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_engine_close() {
        ENGINE_CLOSED.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_output_mixer_open() -> i32 {
        MIXER_OPENED.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_output_mixer_close() {
        MIXER_CLOSED.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_output_create_player(
        object: *mut *mut c_void,
        audio_source: *mut c_void,
    ) -> i32 {
        assert_eq!(audio_source, AUDIO_SOURCE_PTR);
        PLAYER_CREATED.fetch_add(1, Ordering::SeqCst);
        *object = OBJECT_PTR;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_input_create_recorder(
        object: *mut *mut c_void,
        audio_source: *mut c_void,
        audio_sink: *mut c_void,
    ) -> i32 {
        assert_eq!(audio_source, AUDIO_SOURCE_PTR);
        assert_eq!(audio_sink, AUDIO_SINK_PTR);
        RECORDER_CREATED.fetch_add(1, Ordering::SeqCst);
        *object = OBJECT_PTR;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_get_configuration(
        _object: *mut c_void,
        config: *mut *mut c_void,
    ) -> i32 {
        *config = CONFIG_PTR;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_realize(_object: *mut c_void) -> i32 {
        OBJECT_REALIZED.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_destroy(_object: *mut c_void) {
        OBJECT_DESTROYED.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn fake_get_play(_object: *mut c_void, play: *mut *mut c_void) -> i32 {
        *play = PLAY_PTR;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_get_record(_object: *mut c_void, record: *mut *mut c_void) -> i32 {
        *record = RECORD_PTR;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_get_queue(_object: *mut c_void, queue: *mut *mut c_void) -> i32 {
        *queue = QUEUE_PTR;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_set_performance_mode(_config: *mut c_void, mode: i32) -> i32 {
        assert_eq!(mode, OPENSL_PERFORMANCE_LATENCY);
        PERFORMANCE_MODE_SET.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_get_performance_mode(_config: *mut c_void, mode: *mut i32) -> i32 {
        *mode = OPENSL_PERFORMANCE_LATENCY;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_set_stream_type(_config: *mut c_void, stream_type: i32) -> i32 {
        assert_eq!(stream_type, OPENSL_STREAM_MEDIA);
        STREAM_TYPE_SET.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_set_recording_preset(_config: *mut c_void, preset: i32) -> i32 {
        RECORDING_PRESET_SET.fetch_add(1, Ordering::SeqCst);
        if preset == OPENSL_RECORDING_PRESET_GENERIC {
            return SL_RESULT_INTERNAL_ERROR;
        }
        assert_eq!(preset, OPENSL_RECORDING_PRESET_VOICE_RECOGNITION);
        FALLBACK_RECORDING_PRESET_SET.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_register_callback(
        _queue: *mut c_void,
        callback: OboeRustOpenSLESQueueCallback,
        user_data: *mut c_void,
    ) -> i32 {
        assert!(callback.is_some());
        CALLBACK_REGISTERED.fetch_add(1, Ordering::SeqCst);
        CALLBACK_USER_DATA_MATCHED.store(user_data == USER_DATA_PTR, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_enqueue(
        _queue: *mut c_void,
        buffer: *mut c_void,
        num_bytes: i32,
    ) -> i32 {
        assert!(!buffer.is_null());
        assert_eq!(num_bytes, 16);
        QUEUE_ENQUEUED.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_clear(_queue: *mut c_void) -> i32 {
        QUEUE_CLEARED.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_depth(_queue: *mut c_void) -> i32 {
        2
    }

    unsafe extern "C" fn fake_play_set_state(_play: *mut c_void, state: i32) -> i32 {
        assert_eq!(state, PLAYING_STATE);
        PLAY_STATE_SET.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_play_get_position(
        _play: *mut c_void,
        position_millis: *mut i32,
    ) -> i32 {
        *position_millis = 123;
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_record_set_state(_record: *mut c_void, state: i32) -> i32 {
        assert_eq!(state, RECORDING_STATE);
        RECORD_STATE_SET.fetch_add(1, Ordering::SeqCst);
        SL_RESULT_SUCCESS
    }

    unsafe extern "C" fn fake_record_get_position(
        _record: *mut c_void,
        position_millis: *mut i32,
    ) -> i32 {
        *position_millis = 456;
        SL_RESULT_SUCCESS
    }

    extern "C" fn fake_queue_callback(_user_data: *mut c_void) {}

    fn platform() -> OboeRustOpenSLESPlatform {
        let mut platform = OboeRustOpenSLESPlatform::empty();
        platform.engine_open = Some(fake_engine_open);
        platform.engine_close = Some(fake_engine_close);
        platform.output_mixer_open = Some(fake_output_mixer_open);
        platform.output_mixer_close = Some(fake_output_mixer_close);
        platform.output_create_player = Some(fake_output_create_player);
        platform.input_create_recorder = Some(fake_input_create_recorder);
        platform.object_get_android_configuration = Some(fake_get_configuration);
        platform.object_realize = Some(fake_realize);
        platform.object_destroy = Some(fake_destroy);
        platform.object_get_play = Some(fake_get_play);
        platform.object_get_record = Some(fake_get_record);
        platform.object_get_simple_buffer_queue = Some(fake_get_queue);
        platform.configuration_set_performance_mode = Some(fake_set_performance_mode);
        platform.configuration_get_performance_mode = Some(fake_get_performance_mode);
        platform.configuration_set_stream_type = Some(fake_set_stream_type);
        platform.configuration_set_recording_preset = Some(fake_set_recording_preset);
        platform.queue_register_callback = Some(fake_register_callback);
        platform.queue_enqueue = Some(fake_enqueue);
        platform.queue_clear = Some(fake_clear);
        platform.queue_get_depth = Some(fake_depth);
        platform.play_set_state = Some(fake_play_set_state);
        platform.play_get_position_millis = Some(fake_play_get_position);
        platform.record_set_state = Some(fake_record_set_state);
        platform.record_get_position_millis = Some(fake_record_get_position);
        platform
    }

    fn common_settings() -> OboeRustOpenSLESCommonSettings {
        OboeRustOpenSLESCommonSettings {
            sdk_version: SDK_P,
            android_api_n_mr1: API_N_MR1,
            android_api_o_mr1: API_O_MR1,
            opensl_performance_mode: OPENSL_PERFORMANCE_LATENCY,
            opensl_performance_none: OPENSL_PERFORMANCE_NONE,
            opensl_performance_latency: OPENSL_PERFORMANCE_LATENCY,
            opensl_performance_latency_effects: OPENSL_PERFORMANCE_LATENCY_EFFECTS,
            opensl_performance_power_saving: OPENSL_PERFORMANCE_POWER_SAVING,
            oboe_performance_none: PERFORMANCE_MODE_NONE,
            oboe_performance_low_latency: PERFORMANCE_MODE_LOW_LATENCY,
            oboe_performance_power_saving: PERFORMANCE_MODE_POWER_SAVING,
            queue_callback: Some(fake_queue_callback),
            queue_callback_user_data: USER_DATA_PTR,
        }
    }

    #[test]
    fn output_backend_owns_object_graph_and_forwards_runtime_calls() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        let platform = platform();
        let settings = OboeRustOpenSLESOutputSettings {
            common: common_settings(),
            audio_source: AUDIO_SOURCE_PTR,
            opensl_stream_type: OPENSL_STREAM_MEDIA,
        };
        let mut properties = OboeRustOpenSLESOutputProperties::default();

        let backend =
            unsafe { oboe_rust_opensles_output_open(&platform, &settings, &mut properties) };

        assert!(!backend.is_null());
        assert_eq!(properties.result, SL_RESULT_SUCCESS);
        assert_eq!(properties.raw_object, OBJECT_PTR);
        assert_eq!(properties.raw_play, PLAY_PTR);
        assert_eq!(properties.raw_queue, QUEUE_PTR);
        assert_eq!(
            properties.resolved_performance_mode,
            PERFORMANCE_MODE_LOW_LATENCY
        );
        assert_eq!(ENGINE_OPENED.load(Ordering::SeqCst), 1);
        assert_eq!(MIXER_OPENED.load(Ordering::SeqCst), 1);
        assert_eq!(PLAYER_CREATED.load(Ordering::SeqCst), 1);
        assert_eq!(PERFORMANCE_MODE_SET.load(Ordering::SeqCst), 1);
        assert_eq!(STREAM_TYPE_SET.load(Ordering::SeqCst), 1);
        assert_eq!(OBJECT_REALIZED.load(Ordering::SeqCst), 1);
        assert_eq!(CALLBACK_REGISTERED.load(Ordering::SeqCst), 1);
        assert!(CALLBACK_USER_DATA_MATCHED.load(Ordering::SeqCst));

        let mut buffer = [0_u8; 16];
        assert_eq!(
            unsafe {
                oboe_rust_opensles_output_enqueue(backend, buffer.as_mut_ptr().cast::<c_void>(), 16)
            },
            SL_RESULT_SUCCESS
        );
        assert_eq!(QUEUE_ENQUEUED.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_opensles_output_get_buffer_depth(backend) },
            2
        );
        assert_eq!(
            unsafe { oboe_rust_opensles_output_set_play_state(backend, PLAYING_STATE) },
            SL_RESULT_SUCCESS
        );
        assert_eq!(PLAY_STATE_SET.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_opensles_output_clear_queue(backend) },
            SL_RESULT_SUCCESS
        );
        assert_eq!(QUEUE_CLEARED.load(Ordering::SeqCst), 1);
        let mut position = 0;
        assert_eq!(
            unsafe { oboe_rust_opensles_output_get_position_millis(backend, &mut position) },
            SL_RESULT_SUCCESS
        );
        assert_eq!(position, 123);

        assert_eq!(
            unsafe { oboe_rust_opensles_output_destroy(backend) },
            SL_RESULT_SUCCESS
        );
        assert_eq!(OBJECT_DESTROYED.load(Ordering::SeqCst), 1);
        assert_eq!(MIXER_CLOSED.load(Ordering::SeqCst), 1);
        assert_eq!(ENGINE_CLOSED.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn input_backend_owns_recorder_and_applies_recording_preset_fallback() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        let platform = platform();
        let settings = OboeRustOpenSLESInputSettings {
            common: common_settings(),
            audio_source: AUDIO_SOURCE_PTR,
            audio_sink: AUDIO_SINK_PTR,
            opensl_recording_preset: OPENSL_RECORDING_PRESET_GENERIC,
            opensl_recording_preset_voice_recognition: OPENSL_RECORDING_PRESET_VOICE_RECOGNITION,
            oboe_input_preset: INPUT_PRESET_GENERIC_VALUE,
            oboe_input_preset_voice_recognition: INPUT_PRESET_VOICE_RECOGNITION_VALUE,
        };
        let mut properties = OboeRustOpenSLESInputProperties::default();

        let backend =
            unsafe { oboe_rust_opensles_input_open(&platform, &settings, &mut properties) };

        assert!(!backend.is_null());
        assert_eq!(properties.result, SL_RESULT_SUCCESS);
        assert_eq!(properties.raw_object, OBJECT_PTR);
        assert_eq!(properties.raw_record, RECORD_PTR);
        assert_eq!(properties.raw_queue, QUEUE_PTR);
        assert_eq!(
            properties.resolved_performance_mode,
            PERFORMANCE_MODE_LOW_LATENCY
        );
        assert_eq!(
            properties.resolved_input_preset,
            INPUT_PRESET_VOICE_RECOGNITION_VALUE
        );
        assert_eq!(ENGINE_OPENED.load(Ordering::SeqCst), 1);
        assert_eq!(RECORDER_CREATED.load(Ordering::SeqCst), 1);
        assert_eq!(RECORDING_PRESET_SET.load(Ordering::SeqCst), 2);
        assert_eq!(FALLBACK_RECORDING_PRESET_SET.load(Ordering::SeqCst), 1);
        assert_eq!(PERFORMANCE_MODE_SET.load(Ordering::SeqCst), 1);
        assert_eq!(OBJECT_REALIZED.load(Ordering::SeqCst), 1);
        assert_eq!(CALLBACK_REGISTERED.load(Ordering::SeqCst), 1);

        let mut buffer = [0_u8; 16];
        assert_eq!(
            unsafe {
                oboe_rust_opensles_input_enqueue(backend, buffer.as_mut_ptr().cast::<c_void>(), 16)
            },
            SL_RESULT_SUCCESS
        );
        assert_eq!(QUEUE_ENQUEUED.load(Ordering::SeqCst), 1);
        assert_eq!(
            unsafe { oboe_rust_opensles_input_get_buffer_depth(backend) },
            2
        );
        assert_eq!(
            unsafe { oboe_rust_opensles_input_set_record_state(backend, RECORDING_STATE) },
            SL_RESULT_SUCCESS
        );
        assert_eq!(RECORD_STATE_SET.load(Ordering::SeqCst), 1);
        let mut position = 0;
        assert_eq!(
            unsafe { oboe_rust_opensles_input_get_position_millis(backend, &mut position) },
            SL_RESULT_SUCCESS
        );
        assert_eq!(position, 456);

        assert_eq!(
            unsafe { oboe_rust_opensles_input_destroy(backend) },
            SL_RESULT_SUCCESS
        );
        assert_eq!(OBJECT_DESTROYED.load(Ordering::SeqCst), 1);
        assert_eq!(ENGINE_CLOSED.load(Ordering::SeqCst), 1);
    }
}
