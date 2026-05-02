use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::stream::{StreamCore, StreamState};

#[derive(Debug)]
pub struct FakeBackend {
    core: StreamCore,
}

impl AudioBackend for FakeBackend {
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
}
