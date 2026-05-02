#![deny(unsafe_op_in_unsafe_fn)]

pub fn version_code() -> i32 {
    oboe_core::VERSION_CODE
}
