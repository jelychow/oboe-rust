use crate::builder::StreamBuilder;
use crate::error::{Error, Result};
use crate::extensions::{
    CallbackConfig, OffloadDelayPadding, PlaybackParameters, PresentationTimestamp,
};

/// Backend-neutral stream lifecycle states owned by Rust core.
///
/// `StreamCore` currently models the steady-state lifecycle contract. Platform
/// backends may translate platform-specific transitional states into this enum
/// later as real AAudio and OpenSL implementations are added.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamState {
    Uninitialized,
    Open,
    Starting,
    Started,
    Pausing,
    Paused,
    Flushing,
    Flushed,
    Stopping,
    Stopped,
    Closed,
}

/// Minimal stream lifecycle owner shared by backend implementations.
#[derive(Debug)]
pub struct StreamCore {
    state: StreamState,
    callback_config: CallbackConfig,
    offload_delay_padding: OffloadDelayPadding,
    offload_end_of_stream: bool,
    playback_parameters: PlaybackParameters,
    presentation_timestamp: Option<PresentationTimestamp>,
    route_device_id: Option<i32>,
}

impl StreamCore {
    pub fn new_open() -> Self {
        Self {
            state: StreamState::Open,
            callback_config: CallbackConfig::default(),
            offload_delay_padding: OffloadDelayPadding::default(),
            offload_end_of_stream: false,
            playback_parameters: PlaybackParameters::default(),
            presentation_timestamp: None,
            route_device_id: None,
        }
    }

    pub fn new_open_with_builder(builder: &StreamBuilder) -> Result<Self> {
        builder.validate()?;
        Ok(Self {
            state: StreamState::Open,
            callback_config: builder.callback_config,
            offload_delay_padding: builder.offload_delay_padding,
            offload_end_of_stream: false,
            playback_parameters: builder.playback_parameters,
            presentation_timestamp: None,
            route_device_id: None,
        })
    }

    pub fn state(&self) -> StreamState {
        self.state
    }

    pub fn request_start(&mut self) -> Result<()> {
        match self.state {
            StreamState::Closed => Err(Error::Closed),
            StreamState::Started => Ok(()),
            _ => {
                self.state = StreamState::Started;
                Ok(())
            }
        }
    }

    pub fn request_stop(&mut self) -> Result<()> {
        match self.state {
            StreamState::Closed => Err(Error::Closed),
            StreamState::Stopped => Ok(()),
            _ => {
                self.state = StreamState::Stopped;
                Ok(())
            }
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match self.state {
            StreamState::Closed => Err(Error::Closed),
            _ => {
                self.state = StreamState::Closed;
                Ok(())
            }
        }
    }

    pub fn set_callback_config(&mut self, config: CallbackConfig) -> Result<()> {
        self.ensure_open()?;
        config.validate()?;
        self.callback_config = config;
        Ok(())
    }

    pub fn callback_config(&self) -> CallbackConfig {
        self.callback_config
    }

    pub fn set_offload_delay_padding(&mut self, delay_padding: OffloadDelayPadding) -> Result<()> {
        self.ensure_open()?;
        delay_padding.validate()?;
        self.offload_delay_padding = delay_padding;
        Ok(())
    }

    pub fn offload_delay_padding(&self) -> OffloadDelayPadding {
        self.offload_delay_padding
    }

    pub fn set_offload_end_of_stream(&mut self) -> Result<()> {
        self.ensure_open()?;
        self.offload_end_of_stream = true;
        Ok(())
    }

    pub fn is_offload_end_of_stream(&self) -> bool {
        self.offload_end_of_stream
    }

    pub fn set_playback_parameters(&mut self, parameters: PlaybackParameters) -> Result<()> {
        self.ensure_open()?;
        parameters.validate()?;
        self.playback_parameters = parameters;
        Ok(())
    }

    pub fn playback_parameters(&self) -> PlaybackParameters {
        self.playback_parameters
    }

    pub fn set_presentation_timestamp(&mut self, timestamp: PresentationTimestamp) -> Result<()> {
        self.ensure_open()?;
        timestamp.validate()?;
        self.presentation_timestamp = Some(timestamp);
        Ok(())
    }

    pub fn presentation_timestamp(&self) -> Option<PresentationTimestamp> {
        self.presentation_timestamp
    }

    pub fn set_route_device_id(&mut self, device_id: i32) -> Result<()> {
        self.ensure_open()?;
        if device_id < 0 {
            return Err(Error::InvalidArgument);
        }
        self.route_device_id = Some(device_id);
        Ok(())
    }

    pub fn route_device_id(&self) -> Option<i32> {
        self.route_device_id
    }

    fn ensure_open(&self) -> Result<()> {
        if self.state == StreamState::Closed {
            Err(Error::Closed)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::{CallbackConfig, OffloadDelayPadding, PlaybackParameters};

    #[test]
    fn stream_start_stop_close_sequence_is_owned_by_core() {
        let mut stream = StreamCore::new_open();
        assert_eq!(stream.request_start(), Ok(()));
        assert_eq!(stream.state(), StreamState::Started);
        assert_eq!(stream.request_stop(), Ok(()));
        assert_eq!(stream.state(), StreamState::Stopped);
        assert_eq!(stream.close(), Ok(()));
        assert_eq!(stream.state(), StreamState::Closed);
    }

    #[test]
    fn closed_stream_rejects_start() {
        let mut stream = StreamCore::new_open();
        assert_eq!(stream.close(), Ok(()));
        assert_eq!(stream.request_start(), Err(Error::Closed));
    }

    #[test]
    fn repeated_close_is_rejected_after_first_close() {
        let mut stream = StreamCore::new_open();
        assert_eq!(stream.close(), Ok(()));
        assert_eq!(stream.close(), Err(Error::Closed));
    }

    #[test]
    fn closed_stream_rejects_stop() {
        let mut stream = StreamCore::new_open();
        assert_eq!(stream.close(), Ok(()));
        assert_eq!(stream.request_stop(), Err(Error::Closed));
    }

    #[test]
    fn stream_core_stores_extension_state_from_builder() {
        let builder = StreamBuilder {
            callback_config: CallbackConfig {
                data_callback: true,
                frames_per_data_callback: 64,
                ..CallbackConfig::default()
            },
            offload_delay_padding: OffloadDelayPadding {
                delay_in_frames: 7,
                padding_in_frames: 11,
            },
            playback_parameters: PlaybackParameters {
                speed: 1.5,
                ..PlaybackParameters::default()
            },
            ..StreamBuilder::default()
        };

        let stream = StreamCore::new_open_with_builder(&builder).unwrap();
        assert!(stream.callback_config().data_callback);
        assert_eq!(stream.callback_config().frames_per_data_callback, 64);
        assert_eq!(stream.offload_delay_padding().padding_in_frames, 11);
        assert_eq!(stream.playback_parameters().speed, 1.5);
    }

    #[test]
    fn closed_stream_rejects_extension_mutation() {
        let mut stream = StreamCore::new_open();
        assert_eq!(stream.close(), Ok(()));
        assert_eq!(
            stream.set_callback_config(CallbackConfig::default()),
            Err(Error::Closed)
        );
        assert_eq!(
            stream.set_playback_parameters(PlaybackParameters::default()),
            Err(Error::Closed)
        );
        assert_eq!(stream.set_route_device_id(7), Err(Error::Closed));
    }
}
