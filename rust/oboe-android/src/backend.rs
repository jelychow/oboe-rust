use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
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
}
