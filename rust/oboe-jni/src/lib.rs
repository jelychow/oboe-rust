#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;

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

unsafe fn stream_from_handle<'a>(handle: jlong) -> Option<&'a mut NativeStream> {
    if handle == 0 {
        return None;
    }

    let ptr = handle as *mut NativeStream;
    if ptr.is_null() {
        return None;
    }

    unsafe { Some(&mut *ptr) }
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
    let api = api_from_jint(api);
    NativeStream::open(api)
        .map(|stream| Box::into_raw(Box::new(stream)) as jlong)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeRequestStart(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    unsafe { stream_from_handle(handle) }
        .map(|stream| stream.request_start())
        .unwrap_or(-1)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeRequestStop(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    unsafe { stream_from_handle(handle) }
        .map(|stream| stream.request_stop())
        .unwrap_or(-1)
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeGetState(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    unsafe { stream_from_handle(handle) }
        .map(|stream| stream_state_code(stream.state()))
        .unwrap_or(-1)
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

    let ptr = handle as *mut NativeStream;
    if ptr.is_null() {
        return -1;
    }

    let mut stream = unsafe { Box::from_raw(ptr) };
    stream.close()
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
    fn unspecified_api_opens_aaudio_handle_and_runs_lifecycle() {
        let handle = Java_com_google_oboe_AudioStream_nativeOpen(null_mut(), null_mut(), 99);
        assert_ne!(handle, 0);

        {
            let stream = unsafe { stream_from_handle(handle) }.unwrap();
            assert!(matches!(stream, NativeStream::AAudio(_)));
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
    fn opensles_api_selects_opensles_backend() {
        let handle = Java_com_google_oboe_AudioStream_nativeOpen(null_mut(), null_mut(), 2);
        assert_ne!(handle, 0);

        {
            let stream = unsafe { stream_from_handle(handle) }.unwrap();
            assert!(matches!(stream, NativeStream::OpenSLES(_)));
        }

        assert_eq!(
            Java_com_google_oboe_AudioStream_nativeClose(null_mut(), null_mut(), handle),
            0
        );
    }
}
