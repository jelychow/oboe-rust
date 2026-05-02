use crate::error::{Error, Result};

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

#[derive(Debug)]
pub struct StreamCore {
    state: StreamState,
}

impl StreamCore {
    pub fn new_open() -> Self {
        Self {
            state: StreamState::Open,
        }
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
        self.state = StreamState::Closed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
