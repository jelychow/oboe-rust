#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    InvalidState,
    Closed,
    Unavailable,
    BackendUnavailable,
    Internal,
    Unimplemented,
    Platform(i32),
}

impl Error {
    pub fn from_platform_result(result: i32) -> Self {
        if result < 0 {
            Self::Platform(result)
        } else {
            Self::Internal
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_negative_platform_result_to_platform_error() {
        assert_eq!(Error::from_platform_result(-899), Error::Platform(-899));
        assert_eq!(Error::from_platform_result(-1), Error::Platform(-1));
    }

    #[test]
    fn maps_non_negative_platform_result_to_internal() {
        assert_eq!(Error::from_platform_result(0), Error::Internal);
        assert_eq!(Error::from_platform_result(7), Error::Internal);
    }
}
