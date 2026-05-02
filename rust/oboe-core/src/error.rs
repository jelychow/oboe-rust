#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    InvalidState,
    Closed,
    Unavailable,
    BackendUnavailable,
    Internal,
    Unimplemented,
}

pub type Result<T> = core::result::Result<T, Error>;
