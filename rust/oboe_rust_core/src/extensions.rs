const RESULT_OK: i32 = 0;
const RESULT_ERROR_UNAVAILABLE: i32 = -889;

const CALLBACK_CONTINUE: i32 = 0;
const CALLBACK_STOP: i32 = 1;

const UNSPECIFIED: i32 = 0;
const MMAP_POLICY_AUTO: i32 = 2;
const MMAP_POLICY_ALWAYS: i32 = 3;

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_callback_should_launch_stop_thread(
    callback_result: i32,
    workarounds_enabled: bool,
    sdk_version: i32,
    android_api_r: i32,
) -> bool {
    callback_result != CALLBACK_CONTINUE && workarounds_enabled && sdk_version <= android_api_r
}

#[no_mangle]
pub extern "C" fn oboe_rust_aaudio_callback_return_result(
    callback_result: i32,
    workarounds_enabled: bool,
    sdk_version: i32,
    android_api_r: i32,
) -> i32 {
    if callback_result == CALLBACK_CONTINUE {
        return CALLBACK_CONTINUE;
    }
    if oboe_rust_aaudio_callback_should_launch_stop_thread(
        callback_result,
        workarounds_enabled,
        sdk_version,
        android_api_r,
    ) {
        CALLBACK_CONTINUE
    } else {
        CALLBACK_STOP
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_mmap_policy_enabled(policy: i32) -> bool {
    policy == MMAP_POLICY_AUTO || policy == MMAP_POLICY_ALWAYS
}

#[no_mangle]
pub extern "C" fn oboe_rust_mmap_enabled_from_policy(policy: i32, mmap_supported: bool) -> bool {
    if policy == UNSPECIFIED {
        mmap_supported
    } else {
        oboe_rust_mmap_policy_enabled(policy)
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_mmap_unavailable_result() -> i32 {
    RESULT_ERROR_UNAVAILABLE
}

#[no_mangle]
pub extern "C" fn oboe_rust_mmap_load_symbols_result(
    loader_available: bool,
    lib_handle_available: bool,
    stream_is_mmap_available: bool,
    set_mmap_policy_available: bool,
    get_mmap_policy_available: bool,
) -> i32 {
    if loader_available
        && lib_handle_available
        && stream_is_mmap_available
        && set_mmap_policy_available
        && get_mmap_policy_available
    {
        RESULT_OK
    } else {
        RESULT_ERROR_UNAVAILABLE
    }
}
