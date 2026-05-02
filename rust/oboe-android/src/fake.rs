use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::extensions::{
    CallbackConfig, OffloadDelayPadding, PlaybackParameters, PresentationTimestamp,
};
use oboe_core::stream::{StreamCore, StreamState};

#[derive(Debug)]
pub struct FakeBackend {
    core: StreamCore,
}

impl AudioBackend for FakeBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        Ok(Self {
            core: StreamCore::new_open_with_builder(builder)?,
        })
    }

    fn request_start(&mut self) -> Result<()> {
        self.core.request_start()
    }

    fn request_stop(&mut self) -> Result<()> {
        self.core.request_stop()
    }

    fn close(&mut self) -> Result<()> {
        self.core.close()
    }

    fn state(&self) -> StreamState {
        self.core.state()
    }

    fn write_f32(&mut self, audio: &[f32], _timeout_nanos: i64) -> Result<i32> {
        Ok(audio.len() as i32)
    }

    fn read_f32(&mut self, audio: &mut [f32], _timeout_nanos: i64) -> Result<i32> {
        for sample in audio.iter_mut() {
            *sample = 0.0;
        }
        Ok(audio.len() as i32)
    }

    fn set_callback_config(&mut self, config: CallbackConfig) -> Result<()> {
        self.core.set_callback_config(config)
    }

    fn set_offload_delay_padding(&mut self, delay_padding: OffloadDelayPadding) -> Result<()> {
        self.core.set_offload_delay_padding(delay_padding)
    }

    fn set_offload_end_of_stream(&mut self) -> Result<()> {
        self.core.set_offload_end_of_stream()
    }

    fn set_playback_parameters(&mut self, parameters: PlaybackParameters) -> Result<()> {
        self.core.set_playback_parameters(parameters)
    }

    fn set_presentation_timestamp(&mut self, timestamp: PresentationTimestamp) -> Result<()> {
        self.core.set_presentation_timestamp(timestamp)
    }

    fn set_route_device_id(&mut self, device_id: i32) -> Result<()> {
        self.core.set_route_device_id(device_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oboe_core::error::Error;

    #[test]
    fn fake_backend_proves_backend_trait_contract() {
        let mut backend = FakeBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.state(), StreamState::Open);
        assert_eq!(backend.request_start(), Ok(()));
        assert_eq!(backend.state(), StreamState::Started);
        assert_eq!(backend.request_stop(), Ok(()));
        assert_eq!(backend.state(), StreamState::Stopped);
        assert_eq!(backend.close(), Ok(()));
        assert_eq!(backend.state(), StreamState::Closed);
    }

    #[test]
    fn fake_backend_rejects_invalid_builder() {
        let builder = StreamBuilder {
            channel_count: 0,
            ..StreamBuilder::default()
        };

        assert_eq!(
            FakeBackend::open(&builder).unwrap_err(),
            Error::InvalidArgument
        );
    }

    #[test]
    fn fake_backend_reads_and_writes_float_buffers() {
        let mut backend = FakeBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.write_f32(&[0.0, 0.5], 0), Ok(2));
        let mut audio = [1.0, 1.0, 1.0];
        assert_eq!(backend.read_f32(&mut audio, 0), Ok(3));
        assert_eq!(audio, [0.0, 0.0, 0.0]);
    }
}
