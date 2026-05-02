use crate::error::{Error, Result};
use crate::types::{AudioApi, Direction, Format, PerformanceMode, SharingMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamBuilder {
    pub api: AudioApi,
    pub direction: Direction,
    pub sharing_mode: SharingMode,
    pub performance_mode: PerformanceMode,
    pub sample_rate: i32,
    pub channel_count: i32,
    pub format: Format,
    pub frames_per_callback: i32,
    pub buffer_capacity_in_frames: i32,
}

impl Default for StreamBuilder {
    fn default() -> Self {
        Self {
            api: AudioApi::Unspecified,
            direction: Direction::Output,
            sharing_mode: SharingMode::Shared,
            performance_mode: PerformanceMode::None,
            sample_rate: 0,
            channel_count: 2,
            format: Format::Float,
            frames_per_callback: 0,
            buffer_capacity_in_frames: 0,
        }
    }
}

impl StreamBuilder {
    pub fn validate(&self) -> Result<()> {
        if self.sample_rate < 0 {
            return Err(Error::InvalidArgument);
        }
        if self.channel_count <= 0 {
            return Err(Error::InvalidArgument);
        }
        if self.frames_per_callback < 0 {
            return Err(Error::InvalidArgument);
        }
        if self.buffer_capacity_in_frames < 0 {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_builder_is_valid_output_float_stream() {
        let builder = StreamBuilder::default();
        assert_eq!(builder.validate(), Ok(()));
    }

    #[test]
    fn rejects_negative_sample_rate() {
        let builder = StreamBuilder {
            sample_rate: -48_000,
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));
    }

    #[test]
    fn rejects_zero_channel_count() {
        let builder = StreamBuilder {
            channel_count: 0,
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));
    }
}
