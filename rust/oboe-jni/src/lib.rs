#![deny(unsafe_op_in_unsafe_fn)]

#[allow(non_camel_case_types)]
type jint = i32;
#[allow(non_camel_case_types)]
type jclass = *mut core::ffi::c_void;
#[allow(non_camel_case_types)]
type JNIEnv = *mut core::ffi::c_void;

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeVersionCode(
    _env: JNIEnv,
    _class: jclass,
) -> jint {
    oboe_android::version_code()
}
