#![deny(unsafe_op_in_unsafe_fn)]

pub mod backend;
pub mod fake;

pub fn version_code() -> i32 {
    oboe_core::VERSION_CODE
}
