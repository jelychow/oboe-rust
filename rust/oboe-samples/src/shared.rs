const TWO_PI: f32 = core::f32::consts::PI * 2.0;

#[derive(Clone, Debug)]
pub struct Oscillator {
    sample_rate: f32,
    frequency: f32,
    amplitude: f32,
    phase: f32,
    wave_on: bool,
}

impl Oscillator {
    pub fn new(sample_rate: i32, frequency: f32, amplitude: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1) as f32,
            frequency,
            amplitude,
            phase: 0.0,
            wave_on: false,
        }
    }

    pub fn set_wave_on(&mut self, wave_on: bool) {
        self.wave_on = wave_on;
    }

    pub fn render_mono(&mut self, output: &mut [f32]) {
        if !self.wave_on {
            output.fill(0.0);
            return;
        }

        let phase_increment = TWO_PI * self.frequency / self.sample_rate;
        for sample in output {
            *sample = if self.phase <= core::f32::consts::PI {
                -self.amplitude
            } else {
                self.amplitude
            };
            self.phase += phase_increment;
            if self.phase > TWO_PI {
                self.phase -= TWO_PI;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SynthSound {
    sample_rate: f32,
    frequency: f32,
    amplitudes: [f32; 5],
    phase: f32,
    master_amplitude: f32,
    amplitude_scaler: f32,
    trigger: bool,
}

impl SynthSound {
    pub fn new(sample_rate: i32, frequency: f32, amplitude: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1) as f32,
            frequency,
            amplitudes: [
                amplitude * 0.2,
                amplitude,
                amplitude * 0.1,
                amplitude * 0.02,
                amplitude * 0.15,
            ],
            phase: 0.0,
            master_amplitude: 0.0,
            amplitude_scaler: 0.0,
            trigger: false,
        }
    }

    pub fn note_on(&mut self) {
        self.trigger = true;
        self.amplitude_scaler = 0.99999;
    }

    pub fn note_off(&mut self) {
        self.amplitude_scaler = 0.999;
    }

    pub fn render_mono(&mut self, output: &mut [f32]) {
        let phase_increment = TWO_PI * self.frequency / self.sample_rate;
        for sample in output {
            if self.trigger {
                self.trigger = false;
                self.master_amplitude = 1.0;
                self.phase = 0.0;
            } else {
                self.master_amplitude *= self.amplitude_scaler;
            }

            *sample = 0.0;
            if self.master_amplitude >= 0.01 {
                for (harmonic, amplitude) in self.amplitudes.iter().enumerate() {
                    *sample += (self.phase * (harmonic as f32 + 1.0)).sin()
                        * *amplitude
                        * self.master_amplitude;
                }
                self.phase += phase_increment;
                if self.phase > TWO_PI {
                    self.phase -= TWO_PI;
                }
            }
        }
    }
}

pub fn mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(mono.len() * 2);
    for sample in mono {
        stereo.push(*sample);
        stereo.push(*sample);
    }
    stereo
}

pub fn mix_mono_tracks(tracks: &[Vec<f32>]) -> Vec<f32> {
    let frame_count = tracks.iter().map(Vec::len).max().unwrap_or(0);
    let mut output = vec![0.0; frame_count];
    for track in tracks {
        for (index, sample) in track.iter().enumerate() {
            output[index] += *sample;
        }
    }
    output
}

pub fn interleave_channel_tracks(channels: &[Vec<f32>], frame_count: usize) -> Vec<f32> {
    if channels.is_empty() {
        return Vec::new();
    }

    let mut output = vec![0.0; frame_count * channels.len()];
    for (channel, track) in channels.iter().enumerate() {
        for frame in 0..frame_count.min(track.len()) {
            output[frame * channels.len() + channel] = track[frame];
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_oscillator_renders_silence_until_tapped() {
        let mut oscillator = Oscillator::new(48_000, 440.0, 0.5);
        let mut output = [1.0; 3];
        oscillator.render_mono(&mut output);
        assert_eq!(output, [0.0, 0.0, 0.0]);

        oscillator.set_wave_on(true);
        oscillator.render_mono(&mut output);
        assert_eq!(output[0], -0.5);
    }

    #[test]
    fn mono_tracks_mix_by_summing_samples() {
        assert_eq!(
            mix_mono_tracks(&[vec![0.25, 0.5], vec![0.75, -0.25]]),
            vec![1.0, 0.25]
        );
    }
}
