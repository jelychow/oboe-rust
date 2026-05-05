use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataCallbackResult {
    Continue,
    Stop,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CallbackConfig {
    pub data_callback: bool,
    pub partial_data_callback: bool,
    pub presentation_callback: bool,
    pub routing_callback: bool,
    pub frames_per_data_callback: i32,
}

impl CallbackConfig {
    pub fn validate(&self) -> Result<()> {
        if self.frames_per_data_callback < 0 {
            return Err(Error::InvalidArgument);
        }
        if self.data_callback && self.partial_data_callback {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FallbackMode {
    Default,
    Mute,
    Fail,
}

impl TryFrom<i32> for FallbackMode {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::Default),
            1 => Ok(Self::Mute),
            2 => Ok(Self::Fail),
            _ => Err(Error::InvalidArgument),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchMode {
    Default,
    Voice,
}

impl TryFrom<i32> for StretchMode {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::Default),
            1 => Ok(Self::Voice),
            _ => Err(Error::InvalidArgument),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaybackParameters {
    pub fallback_mode: FallbackMode,
    pub stretch_mode: StretchMode,
    pub pitch: f32,
    pub speed: f32,
}

impl Default for PlaybackParameters {
    fn default() -> Self {
        Self {
            fallback_mode: FallbackMode::Default,
            stretch_mode: StretchMode::Default,
            pitch: 1.0,
            speed: 1.0,
        }
    }
}

impl PlaybackParameters {
    pub fn validate(&self) -> Result<()> {
        if !self.pitch.is_finite()
            || !self.speed.is_finite()
            || !(0.25..=4.0).contains(&self.pitch)
            || !(0.01..=20.0).contains(&self.speed)
        {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OffloadDelayPadding {
    pub delay_in_frames: i32,
    pub padding_in_frames: i32,
}

impl OffloadDelayPadding {
    pub fn validate(&self) -> Result<()> {
        if self.delay_in_frames < 0 || self.padding_in_frames < 0 {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationTimestamp {
    pub frame_position: i64,
    pub timestamp_nanos: i64,
}

impl PresentationTimestamp {
    pub fn validate(&self) -> Result<()> {
        if self.frame_position < 0 || self.timestamp_nanos < 0 {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}
