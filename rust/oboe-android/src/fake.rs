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
    sample_rate: i32,
    channel_count: i32,
    session_id: i32,
    frames_read: i64,
    frames_written: i64,
    frames_per_burst: i32,
    buffer_capacity_in_frames: i32,
    buffer_size_in_frames: i32,
    xrun_count: i32,
    last_error: i32,
}

impl AudioBackend for FakeBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        let frames_per_burst = if builder.frames_per_callback > 0 {
            builder.frames_per_callback
        } else {
            192
        };
        let buffer_capacity_in_frames = if builder.buffer_capacity_in_frames > 0 {
            builder.buffer_capacity_in_frames
        } else {
            frames_per_burst * 4
        };
        Ok(Self {
            core: StreamCore::new_open_with_builder(builder)?,
            sample_rate: if builder.sample_rate > 0 {
                builder.sample_rate
            } else {
                48_000
            },
            channel_count: builder.channel_count,
            session_id: builder.session_id,
            frames_read: 0,
            frames_written: 0,
            frames_per_burst,
            buffer_capacity_in_frames,
            buffer_size_in_frames: buffer_capacity_in_frames,
            xrun_count: 0,
            last_error: 0,
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
        self.frames_written += complete_frames_from_samples(audio.len(), self.channel_count)?;
        Ok(audio.len() as i32)
    }

    fn read_f32(&mut self, audio: &mut [f32], _timeout_nanos: i64) -> Result<i32> {
        for sample in audio.iter_mut() {
            *sample = 0.0;
        }
        self.frames_read += complete_frames_from_samples(audio.len(), self.channel_count)?;
        Ok(audio.len() as i32)
    }

    fn get_timestamp(&self) -> Result<PresentationTimestamp> {
        let frame_position = self.frames_written.max(self.frames_read);
        Ok(PresentationTimestamp {
            frame_position,
            timestamp_nanos: frame_position * 1_000_000_000_i64 / i64::from(self.sample_rate),
        })
    }

    fn get_frames_read(&self) -> Result<i64> {
        Ok(self.frames_read)
    }

    fn get_frames_written(&self) -> Result<i64> {
        Ok(self.frames_written)
    }

    fn get_xrun_count(&self) -> Result<i32> {
        Ok(self.xrun_count)
    }

    fn get_frames_per_burst(&self) -> Result<i32> {
        Ok(self.frames_per_burst)
    }

    fn get_session_id(&self) -> Result<i32> {
        Ok(self.session_id)
    }

    fn get_buffer_size_in_frames(&self) -> Result<i32> {
        Ok(self.buffer_size_in_frames)
    }

    fn set_buffer_size_in_frames(&mut self, frames: i32) -> Result<i32> {
        if frames <= 0 {
            return Err(oboe_core::error::Error::InvalidArgument);
        }
        self.buffer_size_in_frames = frames.min(self.buffer_capacity_in_frames);
        Ok(self.buffer_size_in_frames)
    }

    fn get_buffer_capacity_in_frames(&self) -> Result<i32> {
        Ok(self.buffer_capacity_in_frames)
    }

    fn get_and_clear_last_error(&mut self) -> Result<i32> {
        let error = self.last_error;
        self.last_error = 0;
        Ok(error)
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

fn complete_frames_from_samples(sample_count: usize, channel_count: i32) -> Result<i64> {
    let channel_count =
        usize::try_from(channel_count).map_err(|_| oboe_core::error::Error::InvalidArgument)?;
    if channel_count == 0 {
        return Err(oboe_core::error::Error::InvalidArgument);
    }
    i64::try_from(sample_count / channel_count)
        .map_err(|_| oboe_core::error::Error::InvalidArgument)
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
