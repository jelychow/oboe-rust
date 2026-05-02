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
    if sample == i16::MIN {
        -1.0
    } else {
        sample as f32 / i16::MAX as f32
    }
}

pub fn float_to_i16(sample: f32) -> i16 {
    let clipped = sample.clamp(-1.0, 1.0);
    if clipped <= -1.0 {
        i16::MIN
    } else {
        (clipped * i16::MAX as f32).round() as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_sample_sizes() {
        assert_eq!(bytes_per_sample(Format::I16), 2);
        assert_eq!(bytes_per_sample(Format::I24), 3);
        assert_eq!(bytes_per_sample(Format::I32), 4);
        assert_eq!(bytes_per_sample(Format::Float), 4);
    }

    #[test]
    fn converts_i16_and_float_with_clipping() {
        assert_eq!(float_to_i16(2.0), i16::MAX);
        assert_eq!(float_to_i16(-2.0), i16::MIN);
        assert!((i16_to_float(i16::MAX) - 1.0).abs() < 0.0001);
    }
}
