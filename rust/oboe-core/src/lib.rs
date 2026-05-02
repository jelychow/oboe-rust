#![deny(unsafe_op_in_unsafe_fn)]

pub mod builder;
pub mod error;
pub mod fifo;
pub mod format;
pub mod resampler;
pub mod stream;
pub mod types;

pub const VERSION_CODE: i32 = 1;
