use crate::shared::{mix_mono_tracks, mono_to_stereo, Oscillator};

const NUM_OSCILLATORS: usize = 100;
const BASE_FREQUENCY: f32 = 116.0;
const FREQUENCY_DIVISOR: f32 = 33.0;
const AMPLITUDE: f32 = 0.009;

#[derive(Debug)]
pub struct MegaDroneSynth {
    oscillators: Vec<Oscillator>,
    channel_count: usize,
}

impl MegaDroneSynth {
    pub fn new(sample_rate: i32, channel_count: usize) -> Self {
        let oscillators = (0..NUM_OSCILLATORS)
            .map(|index| {
                Oscillator::new(
                    sample_rate,
                    BASE_FREQUENCY + (index as f32 / FREQUENCY_DIVISOR),
                    AMPLITUDE,
                )
            })
            .collect();

        Self {
            oscillators,
            channel_count: channel_count.max(1),
        }
    }

    pub fn tap(&mut self, is_on: bool) {
        for oscillator in &mut self.oscillators {
            oscillator.set_wave_on(is_on);
        }
    }

    pub fn render(&mut self, frame_count: usize) -> Vec<f32> {
        let mut tracks = Vec::with_capacity(self.oscillators.len());
        for oscillator in &mut self.oscillators {
            let mut mono = vec![0.0; frame_count];
            oscillator.render_mono(&mut mono);
            tracks.push(mono);
        }

        let mono = mix_mono_tracks(&tracks);
        if self.channel_count == 2 {
            mono_to_stereo(&mono)
        } else {
            mono
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mega_drone_sums_one_hundred_square_oscillators() {
        let mut drone = MegaDroneSynth::new(48_000, 1);
        drone.tap(true);
        let audio = drone.render(1);
        assert!((audio[0] - -0.9).abs() < 0.0001);
    }
}
