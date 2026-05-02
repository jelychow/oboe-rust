use crate::types::Format;

pub fn bytes_per_sample(format: Format) -> usize {
    match format {
        Format::Unspecified => 0,
        Format::I16 => 2,
        Format::I24 => 3,
        Format::I32 | Format::Float => 4,
    }
}

pub fn i16_to_float(sample: i16) -> f32 {
    sample as f32 * (1.0 / 32_768.0)
}

pub fn float_to_i16(sample: f32) -> i16 {
    let scaled = (sample * 32_768.0) as i32;
    scaled.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_sample_sizes() {
        assert_eq!(bytes_per_sample(Format::Unspecified), 0);
        assert_eq!(bytes_per_sample(Format::I16), 2);
        assert_eq!(bytes_per_sample(Format::I24), 3);
        assert_eq!(bytes_per_sample(Format::I32), 4);
        assert_eq!(bytes_per_sample(Format::Float), 4);
    }

    #[test]
    fn converts_i16_and_float_with_clipping() {
        assert_eq!(i16_to_float(i16::MIN), -1.0);
        assert!((i16_to_float(-32_767) - (-32_767.0 / 32_768.0)).abs() < f32::EPSILON);
        assert_eq!(float_to_i16(2.0), i16::MAX);
        assert_eq!(float_to_i16(-2.0), i16::MIN);
        assert_eq!(float_to_i16(-1.0), i16::MIN);
        assert_eq!(float_to_i16(0.5), 16_384);
        assert_eq!(float_to_i16(1.0), i16::MAX);
        assert!((i16_to_float(i16::MAX) - (32_767.0 / 32_768.0)).abs() < f32::EPSILON);
    }
}
