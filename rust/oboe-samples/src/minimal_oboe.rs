#[derive(Debug)]
pub struct SimpleNoiseMaker {
    channel_count: usize,
    state: u64,
}

impl SimpleNoiseMaker {
    pub fn new(channel_count: usize, seed: u64) -> Self {
        Self {
            channel_count: channel_count.max(1),
            state: seed.max(1),
        }
    }

    pub fn render(&mut self, frame_count: usize) -> Vec<f32> {
        let sample_count = frame_count * self.channel_count;
        (0..sample_count)
            .map(|_| self.next_noise_sample())
            .collect()
    }

    fn next_noise_sample(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let unit = ((self.state >> 40) as u32) as f32 / (1u32 << 24) as f32;
        (unit - 0.5) * 0.6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_oboe_noise_is_bounded_and_interleaved() {
        let mut noise = SimpleNoiseMaker::new(2, 7);
        let samples = noise.render(16);
        assert_eq!(samples.len(), 32);
        assert!(samples.iter().all(|sample| (-0.3..=0.3).contains(sample)));
    }
}
