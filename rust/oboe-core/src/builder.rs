use crate::error::{Error, Result};
use crate::extensions::{CallbackConfig, OffloadDelayPadding, PlaybackParameters};
use crate::types::{AudioApi, Direction, Format, PerformanceMode, SharingMode};

#[derive(Clone, Debug, PartialEq)]
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
    pub callback_config: CallbackConfig,
    pub offload_delay_padding: OffloadDelayPadding,
    pub playback_parameters: PlaybackParameters,
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
            callback_config: CallbackConfig::default(),
            offload_delay_padding: OffloadDelayPadding::default(),
            playback_parameters: PlaybackParameters::default(),
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
        self.callback_config.validate()?;
        self.offload_delay_padding.validate()?;
        self.playback_parameters.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::{
        CallbackConfig, FallbackMode, OffloadDelayPadding, PlaybackParameters, StretchMode,
    };

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

    #[test]
    fn rejects_negative_frames_per_callback() {
        let builder = StreamBuilder {
            frames_per_callback: -192,
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));
    }

    #[test]
    fn rejects_negative_buffer_capacity_in_frames() {
        let builder = StreamBuilder {
            buffer_capacity_in_frames: -256,
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));
    }

    #[test]
    fn builder_accepts_callback_and_extension_settings_owned_by_rust() {
        let builder = StreamBuilder {
            callback_config: CallbackConfig {
                data_callback: false,
                partial_data_callback: true,
                presentation_callback: true,
                routing_callback: true,
                frames_per_data_callback: 96,
            },
            offload_delay_padding: OffloadDelayPadding {
                delay_in_frames: 12,
                padding_in_frames: 34,
            },
            playback_parameters: PlaybackParameters {
                fallback_mode: FallbackMode::Mute,
                stretch_mode: StretchMode::Voice,
                pitch: 1.25,
                speed: 0.75,
            },
            ..StreamBuilder::default()
        };

        assert_eq!(builder.validate(), Ok(()));
        assert!(builder.callback_config.partial_data_callback);
        assert_eq!(builder.offload_delay_padding.delay_in_frames, 12);
        assert_eq!(
            builder.playback_parameters.fallback_mode,
            FallbackMode::Mute
        );
    }

    #[test]
    fn builder_rejects_invalid_callback_and_extension_settings() {
        let builder = StreamBuilder {
            callback_config: CallbackConfig {
                frames_per_data_callback: -1,
                ..CallbackConfig::default()
            },
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));

        let builder = StreamBuilder {
            offload_delay_padding: OffloadDelayPadding {
                delay_in_frames: -1,
                ..OffloadDelayPadding::default()
            },
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));

        let builder = StreamBuilder {
            playback_parameters: PlaybackParameters {
                speed: 0.0,
                ..PlaybackParameters::default()
            },
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));
    }
}
