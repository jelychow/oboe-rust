#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use oboe_android::aaudio::AAudioBackend;
use oboe_android::backend::AudioBackend;
use oboe_android::opensles::OpenSLESBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::stream::StreamState;
use oboe_core::types::AudioApi;

#[allow(non_camel_case_types)]
type jint = i32;
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
    fn open(api: AudioApi) -> Option<Self> {
        let builder = StreamBuilder {
            api,
            ..StreamBuilder::default()
        };

        let stream = match api {
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
    streams: HashMap<jlong, NativeStream>,
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
        self.streams.insert(handle, stream);
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

fn with_stream_mut(handle: jlong, f: impl FnOnce(&mut NativeStream) -> jint) -> jint {
    if handle == 0 {
        return -1;
    }

    let mut registry = lock_registry();
    registry.streams.get_mut(&handle).map(f).unwrap_or(-1)
}

fn with_stream(handle: jlong, f: impl FnOnce(&NativeStream) -> jint) -> jint {
    if handle == 0 {
        return -1;
    }

    let registry = lock_registry();
    registry.streams.get(&handle).map(f).unwrap_or(-1)
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
) -> jlong {
    let requested_api = api_from_jint(api);
    let selected_api = selected_backend_api(requested_api);

    NativeStream::open(selected_api)
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

    stream.map(|mut stream| stream.close()).unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr::null_mut;

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
        let handle = Java_com_google_oboe_AudioStream_nativeOpen(null_mut(), null_mut(), 99);
        assert_ne!(handle, 0);

        {
            let registry = lock_registry();
            let stream = registry.streams.get(&handle).unwrap();
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
        let handle = Java_com_google_oboe_AudioStream_nativeOpen(null_mut(), null_mut(), 1);
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
        let handle = Java_com_google_oboe_AudioStream_nativeOpen(null_mut(), null_mut(), 2);
        assert_ne!(handle, 0);

        {
            let registry = lock_registry();
            let stream = registry.streams.get(&handle).unwrap();
            assert_eq!(stream.backend_api(), AudioApi::OpenSLES);
        }

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            0
        );
    }
}
