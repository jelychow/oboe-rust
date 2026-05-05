#![deny(unsafe_op_in_unsafe_fn)]

pub mod aaudio;
pub mod backend;
pub mod fake;
pub mod opensles;

pub fn version_code() -> i32 {
    oboe_core::VERSION_CODE
}
