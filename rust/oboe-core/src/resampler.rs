use crate::error::{Error, Result};

pub fn linear_interpolate(previous: f32, current: f32, fraction: f32) -> Result<f32> {
    if !(0.0..=1.0).contains(&fraction) {
        return Err(Error::InvalidArgument);
    }
    Ok(previous + (current - previous) * fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_interval_endpoints() {
        assert_eq!(linear_interpolate(2.0, 6.0, 0.0), Ok(2.0));
        assert_eq!(linear_interpolate(2.0, 6.0, 1.0), Ok(6.0));
    }

    #[test]
    fn interpolates_midpoint() {
        assert_eq!(linear_interpolate(2.0, 6.0, 0.5), Ok(4.0));
    }

    #[test]
    fn rejects_fraction_outside_unit_interval() {
        assert_eq!(
            linear_interpolate(2.0, 6.0, -0.1),
            Err(Error::InvalidArgument)
        );
        assert_eq!(
            linear_interpolate(2.0, 6.0, 1.1),
            Err(Error::InvalidArgument)
        );
    }

    #[test]
    fn rejects_nan_fraction() {
        assert_eq!(
            linear_interpolate(2.0, 6.0, f32::NAN),
            Err(Error::InvalidArgument)
        );
    }
}
