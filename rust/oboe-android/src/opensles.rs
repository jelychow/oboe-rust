use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::stream::{StreamCore, StreamState};

#[derive(Debug)]
pub struct OpenSLESBackend {
    core: StreamCore,
}

impl AudioBackend for OpenSLESBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        builder.validate()?;
        Ok(Self {
            core: StreamCore::new_open(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use oboe_core::error::Error;

    #[test]
    fn opensl_backend_supports_core_lifecycle_before_real_ffi() {
        let mut backend = OpenSLESBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.state(), StreamState::Open);
        assert_eq!(backend.request_start(), Ok(()));
        assert_eq!(backend.state(), StreamState::Started);
        assert_eq!(backend.request_stop(), Ok(()));
        assert_eq!(backend.state(), StreamState::Stopped);
        assert_eq!(backend.close(), Ok(()));
        assert_eq!(backend.state(), StreamState::Closed);
        assert_eq!(backend.request_stop(), Err(Error::Closed));
    }

    #[test]
    fn opensl_backend_rejects_invalid_builder() {
        let builder = StreamBuilder {
            channel_count: 0,
            ..StreamBuilder::default()
        };

        assert_eq!(
            OpenSLESBackend::open(&builder).unwrap_err(),
            Error::InvalidArgument
        );
    }
}
