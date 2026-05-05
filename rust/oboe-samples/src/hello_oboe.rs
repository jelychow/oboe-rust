use crate::shared::{interleave_channel_tracks, Oscillator};

#[derive(Debug)]
pub struct HelloOboeSample {
    oscillators: Vec<Oscillator>,
    channel_count: usize,
}

impl HelloOboeSample {
    pub fn new(sample_rate: i32, channel_count: usize) -> Self {
        let channel_count = channel_count.max(1);
        let mut frequency = 440.0;
        let oscillators = (0..channel_count)
            .map(|_| {
                let oscillator = Oscillator::new(sample_rate, frequency, 1.0);
                frequency += 110.0;
                oscillator
            })
            .collect();

        Self {
            oscillators,
            channel_count,
        }
    }

    pub fn tap(&mut self, is_on: bool) {
        for oscillator in &mut self.oscillators {
            oscillator.set_wave_on(is_on);
        }
    }

    pub fn render(&mut self, frame_count: usize) -> Vec<f32> {
        let mut channels = Vec::with_capacity(self.channel_count);
        for oscillator in &mut self.oscillators {
            let mut mono = vec![0.0; frame_count];
            oscillator.render_mono(&mut mono);
            channels.push(mono);
        }
        interleave_channel_tracks(&channels, frame_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_oboe_renders_one_tapped_tone_per_channel() {
        let mut sample = HelloOboeSample::new(48_000, 2);
        assert_eq!(sample.render(2), vec![0.0, 0.0, 0.0, 0.0]);
        sample.tap(true);
        assert_eq!(sample.render(1), vec![-1.0, -1.0]);
    }
}
