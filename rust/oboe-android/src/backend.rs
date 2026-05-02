use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::extensions::{
    CallbackConfig, OffloadDelayPadding, PlaybackParameters, PresentationTimestamp,
};
use oboe_core::stream::StreamState;

pub trait AudioBackend {
    fn open(builder: &StreamBuilder) -> Result<Self>
    where
        Self: Sized;
    fn request_start(&mut self) -> Result<()>;
    fn request_stop(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn state(&self) -> StreamState;
    fn write_f32(&mut self, audio: &[f32], timeout_nanos: i64) -> Result<i32>;
    fn read_f32(&mut self, audio: &mut [f32], timeout_nanos: i64) -> Result<i32>;
    fn set_callback_config(&mut self, config: CallbackConfig) -> Result<()>;
    fn set_offload_delay_padding(&mut self, delay_padding: OffloadDelayPadding) -> Result<()>;
    fn set_offload_end_of_stream(&mut self) -> Result<()>;
    fn set_playback_parameters(&mut self, parameters: PlaybackParameters) -> Result<()>;
    fn set_presentation_timestamp(&mut self, timestamp: PresentationTimestamp) -> Result<()>;
    fn set_route_device_id(&mut self, device_id: i32) -> Result<()>;
}
