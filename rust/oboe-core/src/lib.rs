#![deny(unsafe_op_in_unsafe_fn)]

pub mod builder;
pub mod capabilities;
pub mod error;
pub mod extensions;
pub mod fifo;
pub mod format;
pub mod resampler;
pub mod stream;
pub mod types;

pub const VERSION_CODE: i32 = 1;

#[cfg(test)]
mod capability_api_tests {
    use super::capabilities::{capability, SupportLevel};

    #[test]
    fn public_capability_api_reports_callback_gap() {
        let capability = capability("stream_callbacks").unwrap();
        assert_eq!(capability.support, SupportLevel::Unsupported);
    }
}
