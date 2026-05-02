#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use oboe_android::aaudio::AAudioBackend;
use oboe_android::backend::AudioBackend;
use oboe_android::opensles::OpenSLESBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::extensions::{
    CallbackConfig, FallbackMode, OffloadDelayPadding, PlaybackParameters, PresentationTimestamp,
    StretchMode,
};
use oboe_core::stream::StreamState;
use oboe_core::types::{AudioApi, Direction, Format, PerformanceMode, SharingMode};

#[allow(non_camel_case_types)]
type jboolean = u8;
#[allow(non_camel_case_types)]
type jint = i32;
#[allow(non_camel_case_types)]
type jfloat = f32;
#[allow(non_camel_case_types)]
type jlong = i64;
#[allow(non_camel_case_types)]
type jobject = *mut c_void;
#[allow(non_camel_case_types)]
type jclass = *mut c_void;
#[allow(non_camel_case_types)]
type JNIEnv = *mut c_void;

enum NativeStream {
    AAudio(AAudioBackend),
    OpenSLES(OpenSLESBackend),
}

impl NativeStream {
    fn open(builder: StreamBuilder) -> Option<Self> {
        let stream = match builder.api {
            AudioApi::AAudio | AudioApi::Unspecified => {
                AAudioBackend::open(&builder).map(Self::AAudio)
            }
            AudioApi::OpenSLES => OpenSLESBackend::open(&builder).map(Self::OpenSLES),
        };

        stream.ok()
    }

    fn request_start(&mut self) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.request_start(),
            Self::OpenSLES(stream) => stream.request_start(),
        })
    }

    fn request_stop(&mut self) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.request_stop(),
            Self::OpenSLES(stream) => stream.request_stop(),
        })
    }

    fn close(&mut self) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.close(),
            Self::OpenSLES(stream) => stream.close(),
        })
    }

    fn state(&self) -> StreamState {
        match self {
            Self::AAudio(stream) => stream.state(),
            Self::OpenSLES(stream) => stream.state(),
        }
    }

    fn set_callback_config(&mut self, config: CallbackConfig) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.set_callback_config(config),
            Self::OpenSLES(stream) => stream.set_callback_config(config),
        })
    }

    fn set_offload_delay_padding(&mut self, delay_padding: OffloadDelayPadding) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.set_offload_delay_padding(delay_padding),
            Self::OpenSLES(stream) => stream.set_offload_delay_padding(delay_padding),
        })
    }

    fn set_offload_end_of_stream(&mut self) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.set_offload_end_of_stream(),
            Self::OpenSLES(stream) => stream.set_offload_end_of_stream(),
        })
    }

    fn set_playback_parameters(&mut self, parameters: PlaybackParameters) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.set_playback_parameters(parameters),
            Self::OpenSLES(stream) => stream.set_playback_parameters(parameters),
        })
    }

    fn set_presentation_timestamp(&mut self, timestamp: PresentationTimestamp) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.set_presentation_timestamp(timestamp),
            Self::OpenSLES(stream) => stream.set_presentation_timestamp(timestamp),
        })
    }

    fn set_route_device_id(&mut self, device_id: jint) -> jint {
        result_code(match self {
            Self::AAudio(stream) => stream.set_route_device_id(device_id),
            Self::OpenSLES(stream) => stream.set_route_device_id(device_id),
        })
    }

    #[cfg(test)]
    fn backend_api(&self) -> AudioApi {
        match self {
            Self::AAudio(_) => AudioApi::AAudio,
            Self::OpenSLES(_) => AudioApi::OpenSLES,
        }
    }
}

#[derive(Default)]
struct HandleRegistry {
    next_handle: jlong,
    streams: HashMap<jlong, Arc<Mutex<NativeStream>>>,
}

impl HandleRegistry {
    fn insert(&mut self, stream: NativeStream) -> jlong {
        let mut handle = if self.next_handle <= 0 {
            1
        } else {
            self.next_handle
        };

        while self.streams.contains_key(&handle) {
            handle = if handle == jlong::MAX { 1 } else { handle + 1 };
        }

        self.next_handle = if handle == jlong::MAX { 1 } else { handle + 1 };
        self.streams.insert(handle, Arc::new(Mutex::new(stream)));
        handle
    }
}

fn registry() -> &'static Mutex<HandleRegistry> {
    static REGISTRY: OnceLock<Mutex<HandleRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HandleRegistry::default()))
}

fn lock_registry() -> MutexGuard<'static, HandleRegistry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn result_code<T>(result: oboe_core::error::Result<T>) -> jint {
    if result.is_ok() {
        0
    } else {
        -1
    }
}

fn api_from_jint(api: jint) -> AudioApi {
    match api {
        1 => AudioApi::AAudio,
        2 => AudioApi::OpenSLES,
        _ => AudioApi::Unspecified,
    }
}

fn direction_from_jint(direction: jint) -> Option<Direction> {
    match direction {
        0 => Some(Direction::Input),
        1 => Some(Direction::Output),
        _ => None,
    }
}

fn sharing_mode_from_jint(sharing_mode: jint) -> Option<SharingMode> {
    match sharing_mode {
        0 => Some(SharingMode::Shared),
        1 => Some(SharingMode::Exclusive),
        _ => None,
    }
}

fn performance_mode_from_jint(performance_mode: jint) -> Option<PerformanceMode> {
    match performance_mode {
        0 => Some(PerformanceMode::None),
        1 => Some(PerformanceMode::PowerSaving),
        2 => Some(PerformanceMode::LowLatency),
        _ => None,
    }
}

fn format_from_jint(format: jint) -> Option<Format> {
    match format {
        0 => Some(Format::Unspecified),
        1 => Some(Format::I16),
        2 => Some(Format::I24),
        3 => Some(Format::I32),
        4 => Some(Format::Float),
        _ => None,
    }
}

fn jboolean_to_bool(value: jboolean) -> bool {
    value != 0
}

fn playback_parameters_from_jni(
    fallback_mode: jint,
    stretch_mode: jint,
    pitch: jfloat,
    speed: jfloat,
) -> oboe_core::error::Result<PlaybackParameters> {
    let parameters = PlaybackParameters {
        fallback_mode: FallbackMode::try_from(fallback_mode)?,
        stretch_mode: StretchMode::try_from(stretch_mode)?,
        pitch,
        speed,
    };
    parameters.validate()?;
    Ok(parameters)
}

fn selected_backend_api(requested: AudioApi) -> AudioApi {
    // Until real backend availability probing lands, unspecified requests use AAudio.
    match requested {
        AudioApi::Unspecified => AudioApi::AAudio,
        AudioApi::AAudio => AudioApi::AAudio,
        AudioApi::OpenSLES => AudioApi::OpenSLES,
    }
}

fn stream_state_code(state: StreamState) -> jint {
    match state {
        StreamState::Uninitialized => 0,
        StreamState::Open => 1,
        StreamState::Starting => 2,
        StreamState::Started => 3,
        StreamState::Pausing => 4,
        StreamState::Paused => 5,
        StreamState::Flushing => 6,
        StreamState::Flushed => 7,
        StreamState::Stopping => 8,
        StreamState::Stopped => 9,
        StreamState::Closed => 10,
    }
}

fn stream_for_handle(handle: jlong) -> Option<Arc<Mutex<NativeStream>>> {
    if handle == 0 {
        return None;
    }
    let registry = lock_registry();
    registry.streams.get(&handle).cloned()
}

fn with_stream_mut(handle: jlong, f: impl FnOnce(&mut NativeStream) -> jint) -> jint {
    stream_for_handle(handle)
        .map(|stream| {
            let mut stream = stream
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            f(&mut stream)
        })
        .unwrap_or(-1)
}

fn with_stream(handle: jlong, f: impl FnOnce(&NativeStream) -> jint) -> jint {
    stream_for_handle(handle)
        .map(|stream| {
            let stream = stream
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            f(&stream)
        })
        .unwrap_or(-1)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeVersionCode(
    _env: JNIEnv,
    _class: jclass,
) -> jint {
    oboe_android::version_code()
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeOpen(
    _env: JNIEnv,
    _class: jclass,
    api: jint,
    direction: jint,
    sharing_mode: jint,
    performance_mode: jint,
    sample_rate: jint,
    channel_count: jint,
    format: jint,
    frames_per_callback: jint,
    buffer_capacity_in_frames: jint,
) -> jlong {
    let requested_api = api_from_jint(api);
    let selected_api = selected_backend_api(requested_api);
    let builder = StreamBuilder {
        api: selected_api,
        direction: match direction_from_jint(direction) {
            Some(direction) => direction,
            None => return 0,
        },
        sharing_mode: match sharing_mode_from_jint(sharing_mode) {
            Some(sharing_mode) => sharing_mode,
            None => return 0,
        },
        performance_mode: match performance_mode_from_jint(performance_mode) {
            Some(performance_mode) => performance_mode,
            None => return 0,
        },
        sample_rate,
        channel_count,
        format: match format_from_jint(format) {
            Some(format) => format,
            None => return 0,
        },
        frames_per_callback,
        buffer_capacity_in_frames,
        ..StreamBuilder::default()
    };

    NativeStream::open(builder)
        .map(|stream| lock_registry().insert(stream))
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeRequestStart(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    with_stream_mut(handle, NativeStream::request_start)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeRequestStop(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    with_stream_mut(handle, NativeStream::request_stop)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeGetState(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    with_stream(handle, |stream| stream_state_code(stream.state()))
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeSetCallbackConfig(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
    data_callback: jboolean,
    partial_data_callback: jboolean,
    presentation_callback: jboolean,
    routing_callback: jboolean,
    frames_per_data_callback: jint,
) -> jint {
    let config = CallbackConfig {
        data_callback: jboolean_to_bool(data_callback),
        partial_data_callback: jboolean_to_bool(partial_data_callback),
        presentation_callback: jboolean_to_bool(presentation_callback),
        routing_callback: jboolean_to_bool(routing_callback),
        frames_per_data_callback,
    };
    with_stream_mut(handle, |stream| stream.set_callback_config(config))
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeSetOffloadDelayPadding(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
    delay_in_frames: jint,
    padding_in_frames: jint,
) -> jint {
    let delay_padding = OffloadDelayPadding {
        delay_in_frames,
        padding_in_frames,
    };
    with_stream_mut(handle, |stream| {
        stream.set_offload_delay_padding(delay_padding)
    })
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeSetOffloadEndOfStream(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    with_stream_mut(handle, NativeStream::set_offload_end_of_stream)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeSetPlaybackParameters(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
    fallback_mode: jint,
    stretch_mode: jint,
    pitch: jfloat,
    speed: jfloat,
) -> jint {
    let parameters = match playback_parameters_from_jni(fallback_mode, stretch_mode, pitch, speed) {
        Ok(parameters) => parameters,
        Err(_) => return -1,
    };
    with_stream_mut(handle, |stream| stream.set_playback_parameters(parameters))
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeSetPresentationTimestamp(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
    frame_position: jlong,
    timestamp_nanos: jlong,
) -> jint {
    let timestamp = PresentationTimestamp {
        frame_position,
        timestamp_nanos,
    };
    with_stream_mut(handle, |stream| {
        stream.set_presentation_timestamp(timestamp)
    })
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeSetRouteDeviceId(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
    device_id: jint,
) -> jint {
    with_stream_mut(handle, |stream| stream.set_route_device_id(device_id))
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeClose(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    if handle == 0 {
        return -1;
    }

    let stream = {
        let mut registry = lock_registry();
        registry.streams.remove(&handle)
    };

    stream
        .map(|stream| {
            let mut stream = stream
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            stream.close()
        })
        .unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr::null_mut;

    fn native_open(api: jint) -> jlong {
        Java_com_google_oboe_AudioStream_nativeOpen(
            null_mut(),
            null_mut(),
            api,
            1,
            0,
            0,
            0,
            2,
            4,
            0,
            0,
        )
    }

    #[test]
    fn version_code_matches_android_crate() {
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeVersionCode(null_mut(), null_mut()),
            oboe_android::version_code()
        );
    }

    #[test]
    fn invalid_handle_returns_error_codes() {
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeRequestStart(null_mut(), null_mut(), 0),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeRequestStop(null_mut(), null_mut(), 0),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeGetState(null_mut(), null_mut(), 0),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), 0),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetCallbackConfig(
                null_mut(),
                null_mut(),
                0,
                1,
                0,
                1,
                1,
                96,
            ),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetOffloadDelayPadding(
                null_mut(),
                null_mut(),
                0,
                12,
                34,
            ),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetOffloadEndOfStream(null_mut(), null_mut(), 0),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetPlaybackParameters(
                null_mut(),
                null_mut(),
                0,
                1,
                1,
                1.0,
                1.0,
            ),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetPresentationTimestamp(
                null_mut(),
                null_mut(),
                0,
                128,
                1024,
            ),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetRouteDeviceId(null_mut(), null_mut(), 0, 7),
            -1
        );
    }

    #[test]
    fn unknown_api_maps_to_unspecified_then_selects_aaudio() {
        assert_eq!(api_from_jint(99), AudioApi::Unspecified);
        assert_eq!(selected_backend_api(api_from_jint(99)), AudioApi::AAudio);
        assert_eq!(
            selected_backend_api(AudioApi::Unspecified),
            AudioApi::AAudio
        );
    }

    #[test]
    fn unspecified_api_opens_aaudio_handle_and_runs_lifecycle() {
        let handle = native_open(99);
        assert_ne!(handle, 0);

        {
            let registry = lock_registry();
            let stream = registry.streams.get(&handle).unwrap();
            let stream = stream.lock().unwrap();
            assert_eq!(stream.backend_api(), AudioApi::AAudio);
        }

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeGetState(null_mut(), null_mut(), handle),
            stream_state_code(StreamState::Open)
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeRequestStart(null_mut(), null_mut(), handle),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeGetState(null_mut(), null_mut(), handle),
            stream_state_code(StreamState::Started)
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeRequestStop(null_mut(), null_mut(), handle),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeGetState(null_mut(), null_mut(), handle),
            stream_state_code(StreamState::Stopped)
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            0
        );
    }

    #[test]
    fn closed_handles_are_removed_and_stay_invalid() {
        let handle = native_open(1);
        assert_ne!(handle, 0);
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeRequestStart(null_mut(), null_mut(), handle),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeRequestStop(null_mut(), null_mut(), handle),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeGetState(null_mut(), null_mut(), handle),
            -1
        );
    }

    #[test]
    fn opensles_api_selects_opensles_backend() {
        let handle = native_open(2);
        assert_ne!(handle, 0);

        {
            let registry = lock_registry();
            let stream = registry.streams.get(&handle).unwrap();
            let stream = stream.lock().unwrap();
            assert_eq!(stream.backend_api(), AudioApi::OpenSLES);
        }

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            0
        );
    }

    #[test]
    fn aaudio_handle_accepts_callback_and_extension_paths() {
        let handle = native_open(1);
        assert_ne!(handle, 0);

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetCallbackConfig(
                null_mut(),
                null_mut(),
                handle,
                0,
                1,
                1,
                1,
                96,
            ),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetOffloadDelayPadding(
                null_mut(),
                null_mut(),
                handle,
                12,
                34,
            ),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetOffloadEndOfStream(
                null_mut(),
                null_mut(),
                handle,
            ),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetPlaybackParameters(
                null_mut(),
                null_mut(),
                handle,
                1,
                1,
                1.25,
                0.75,
            ),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetPresentationTimestamp(
                null_mut(),
                null_mut(),
                handle,
                128,
                1024,
            ),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetRouteDeviceId(
                null_mut(),
                null_mut(),
                handle,
                7,
            ),
            0
        );

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            0
        );
    }

    #[test]
    fn opensles_handle_keeps_callback_path_but_rejects_aaudio_only_extensions() {
        let handle = native_open(2);
        assert_ne!(handle, 0);

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetCallbackConfig(
                null_mut(),
                null_mut(),
                handle,
                1,
                0,
                0,
                0,
                96,
            ),
            0
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetOffloadDelayPadding(
                null_mut(),
                null_mut(),
                handle,
                12,
                34,
            ),
            -1
        );
        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeSetPlaybackParameters(
                null_mut(),
                null_mut(),
                handle,
                1,
                1,
                1.0,
                1.0,
            ),
            -1
        );

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            0
        );
    }

    #[test]
    fn native_open_rejects_invalid_builder_config() {
        let handle = Java_com_google_oboe_AudioStream_nativeOpen(
            null_mut(),
            null_mut(),
            1,
            1,
            0,
            2,
            24_000,
            0,
            4,
            0,
            0,
        );

        assert_eq!(handle, 0);
    }
}
