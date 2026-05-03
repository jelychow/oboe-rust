use oboe_android::aaudio::AAudioBackend;
use oboe_android::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::callback::{AudioCallbackInfo, AudioStreamCallback, RouteChange};
use oboe_core::extensions::DataCallbackResult;
use oboe_core::stream::StreamState;
use oboe_core::types::{Direction, Format, PerformanceMode, SharingMode};
use std::sync::{Arc, Mutex};

#[test]
fn aaudio_backend_exposes_low_latency_diagnostics_and_buffer_tuning() {
    let builder = StreamBuilder {
        direction: Direction::Output,
        sharing_mode: SharingMode::Exclusive,
        performance_mode: PerformanceMode::LowLatency,
        sample_rate: 48_000,
        channel_count: 1,
        format: Format::Float,
        frames_per_callback: 96,
        buffer_capacity_in_frames: 384,
        ..StreamBuilder::default()
    };
    let mut backend = AAudioBackend::open(&builder).unwrap();

    assert_eq!(backend.state(), StreamState::Open);
    assert_eq!(backend.get_frames_per_burst().unwrap(), 96);
    assert_eq!(backend.get_buffer_capacity_in_frames().unwrap(), 384);
    assert_eq!(backend.set_buffer_size_in_frames(192).unwrap(), 192);
    assert_eq!(backend.get_buffer_size_in_frames().unwrap(), 192);
    assert_eq!(backend.get_xrun_count().unwrap(), 0);

    assert_eq!(backend.request_start(), Ok(()));
    assert_eq!(backend.write_f32(&[0.0; 96], 0).unwrap(), 96);
    assert_eq!(backend.get_frames_written().unwrap(), 96);

    let timestamp = backend.get_timestamp().unwrap();
    assert_eq!(timestamp.frame_position, 96);
    assert!(timestamp.timestamp_nanos > 0);
}

#[test]
fn aaudio_callback_receives_explicit_route_updates() {
    struct RouteRecorder {
        routes: Arc<Mutex<Vec<RouteChange>>>,
    }

    impl AudioStreamCallback for RouteRecorder {
        fn on_audio_ready(
            &self,
            _info: AudioCallbackInfo,
            _audio_data: &mut [f32],
        ) -> DataCallbackResult {
            DataCallbackResult::Continue
        }

        fn on_route_changed(&self, route: RouteChange) {
            self.routes.lock().unwrap().push(route);
        }
    }

    let routes = Arc::new(Mutex::new(Vec::new()));
    let mut backend = AAudioBackend::open_with_callback(
        &StreamBuilder {
            sample_rate: 48_000,
            channel_count: 1,
            format: Format::Float,
            ..StreamBuilder::default()
        },
        Box::new(RouteRecorder {
            routes: Arc::clone(&routes),
        }),
    )
    .unwrap();

    assert_eq!(backend.set_route_device_id(42), Ok(()));
    assert_eq!(
        *routes.lock().unwrap(),
        vec![RouteChange {
            device_id: Some(42),
        }]
    );
}
