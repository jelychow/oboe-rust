pub fn process_full_duplex(
    input_samples: &[f32],
    channel_count: usize,
    output_frame_count: usize,
) -> Vec<f32> {
    if channel_count == 0 {
        return Vec::new();
    }

    let output_sample_count = output_frame_count * channel_count;
    let mut output = vec![0.0; output_sample_count];
    let samples_to_process = input_samples.len().min(output_sample_count);
    for index in 0..samples_to_process {
        output[index] = input_samples[index] * 0.95;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_duplex_pass_applies_gain_and_clears_missing_input() {
        assert_eq!(
            process_full_duplex(&[1.0, -0.5], 1, 4),
            vec![0.95, -0.475, 0.0, 0.0]
        );
    }
}
