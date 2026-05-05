use crate::error::Error;
use crate::extensions::DataCallbackResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioCallbackInfo {
    pub num_frames: i32,
    pub channel_count: i32,
    pub sample_rate: i32,
    pub input: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteChange {
    pub device_id: Option<i32>,
}

pub trait AudioStreamCallback: Send + Sync {
    fn on_audio_ready(&self, info: AudioCallbackInfo, audio_data: &mut [f32])
        -> DataCallbackResult;

    fn on_error(&self, _error: Error) {}

    fn on_route_changed(&self, _route: RouteChange) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StopCallback;

    impl AudioStreamCallback for StopCallback {
        fn on_audio_ready(
            &self,
            _info: AudioCallbackInfo,
            _audio_data: &mut [f32],
        ) -> DataCallbackResult {
            DataCallbackResult::Stop
        }
    }

    #[test]
    fn callback_trait_supports_stop_result() {
        let callback = StopCallback;
        let mut audio = [0.0_f32; 2];
        assert_eq!(
            callback.on_audio_ready(
                AudioCallbackInfo {
                    num_frames: 1,
                    channel_count: 2,
                    sample_rate: 48_000,
                    input: false,
                },
                &mut audio,
            ),
            DataCallbackResult::Stop
        );
    }
}
