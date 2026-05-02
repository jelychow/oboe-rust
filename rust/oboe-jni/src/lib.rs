#![deny(unsafe_op_in_unsafe_fn)]

#[allow(non_camel_case_types)]
type jint = i32;

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeVersionCode() -> jint {
    oboe_android::version_code()
}
