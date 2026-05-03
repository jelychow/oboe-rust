use oboe_core::callback::{AudioCallbackInfo, AudioStreamCallback, RouteChange};
use oboe_core::error::Error;
use oboe_core::extensions::DataCallbackResult;

#[derive(Default)]
struct CountingCallback {
    frames_seen: std::sync::Mutex<i32>,
    errors_seen: std::sync::Mutex<Vec<Error>>,
    routes_seen: std::sync::Mutex<Vec<RouteChange>>,
}

impl AudioStreamCallback for CountingCallback {
    fn on_audio_ready(
        &self,
        info: AudioCallbackInfo,
        audio_data: &mut [f32],
    ) -> DataCallbackResult {
        *self.frames_seen.lock().unwrap() += info.num_frames;
        for sample in audio_data {
            *sample = 0.25;
        }
        DataCallbackResult::Continue
    }

    fn on_error(&self, error: Error) {
        self.errors_seen.lock().unwrap().push(error);
    }

    fn on_route_changed(&self, route: RouteChange) {
        self.routes_seen.lock().unwrap().push(route);
    }
}

#[test]
fn callback_contract_carries_realtime_audio_errors_and_routes() {
    let callback = CountingCallback::default();
    let mut audio = [0.0_f32; 8];

    let result = callback.on_audio_ready(
        AudioCallbackInfo {
            num_frames: 4,
            channel_count: 2,
            sample_rate: 48_000,
            input: false,
        },
        &mut audio,
    );
    callback.on_error(Error::Platform(-899));
    callback.on_route_changed(RouteChange { device_id: Some(7) });

    assert_eq!(result, DataCallbackResult::Continue);
    assert_eq!(*callback.frames_seen.lock().unwrap(), 4);
    assert_eq!(audio, [0.25; 8]);
    assert_eq!(
        *callback.errors_seen.lock().unwrap(),
        vec![Error::Platform(-899)]
    );
    assert_eq!(
        *callback.routes_seen.lock().unwrap(),
        vec![RouteChange { device_id: Some(7) }]
    );
}
