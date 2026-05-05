use crate::parselib::WavData;
use oboe_core::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioProperties {
    pub channel_count: usize,
    pub sample_rate: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleBuffer {
    samples: Vec<f32>,
    properties: AudioProperties,
}

impl SampleBuffer {
    pub fn new(samples: Vec<f32>, channel_count: usize, sample_rate: u32) -> Result<Self> {
        if channel_count == 0 || samples.len() % channel_count != 0 {
            return Err(Error::InvalidArgument);
        }
        Ok(Self {
            samples,
            properties: AudioProperties {
                channel_count,
                sample_rate,
            },
        })
    }

    pub fn from_wav_bytes(bytes: &[u8]) -> Result<Self> {
        let wav = WavData::parse(bytes)?;
        Self::new(wav.frames, usize::from(wav.channel_count), wav.sample_rate)
    }

    pub fn properties(&self) -> AudioProperties {
        self.properties
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

#[derive(Clone, Debug)]
pub struct OneShotSampleSource {
    buffer: SampleBuffer,
    cursor: usize,
    playing: bool,
    loop_mode: bool,
    pan: f32,
    gain: f32,
    left_gain: f32,
    right_gain: f32,
}

impl OneShotSampleSource {
    pub fn new(buffer: SampleBuffer, pan: f32) -> Self {
        let mut source = Self {
            buffer,
            cursor: 0,
            playing: false,
            loop_mode: false,
            pan: 0.0,
            gain: 1.0,
            left_gain: 0.5,
            right_gain: 0.5,
        };
        source.set_pan(pan);
        source
    }

    pub fn trigger(&mut self) {
        self.playing = true;
        self.cursor = 0;
    }

    pub fn stop(&mut self, pause: bool) {
        self.playing = false;
        if !pause {
            self.cursor = 0;
        }
    }

    pub fn set_loop_mode(&mut self, loop_mode: bool) {
        self.loop_mode = loop_mode;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn set_play_head_position(&mut self, position: usize) -> Result<()> {
        if position >= self.buffer.sample_count() {
            return Err(Error::InvalidArgument);
        }
        self.cursor = position;
        Ok(())
    }

    pub fn play_head_position(&self) -> usize {
        self.cursor
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
        self.recalculate_gains();
    }

    pub fn pan(&self) -> f32 {
        self.pan
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
        self.recalculate_gains();
    }

    pub fn gain(&self) -> f32 {
        self.gain
    }

    pub fn duration_frames(&self) -> usize {
        self.buffer.sample_count() / self.buffer.properties.channel_count
    }

    pub fn mix_into(&mut self, output: &mut [f32], output_channels: usize, frame_count: usize) {
        if output_channels == 0 || !self.playing {
            return;
        }

        let sample_channels = self.buffer.properties.channel_count;
        let total_samples_needed = frame_count
            .saturating_mul(output_channels)
            .min(output.len());
        let mut samples_processed = 0;

        while samples_processed < total_samples_needed && self.playing {
            let samples_left = self.buffer.sample_count().saturating_sub(self.cursor);
            let frames_left = (total_samples_needed - samples_processed) / output_channels;
            let write_frames = frames_left.min(samples_left / sample_channels);
            if write_frames == 0 {
                break;
            }

            for _ in 0..write_frames {
                match (sample_channels, output_channels) {
                    (1, 1) => {
                        output[samples_processed] += self.buffer.samples[self.cursor] * self.gain;
                        self.cursor += 1;
                        samples_processed += 1;
                    }
                    (1, 2) => {
                        let sample = self.buffer.samples[self.cursor];
                        output[samples_processed] += sample * self.left_gain;
                        output[samples_processed + 1] += sample * self.right_gain;
                        self.cursor += 1;
                        samples_processed += 2;
                    }
                    (2, 1) => {
                        output[samples_processed] += self.buffer.samples[self.cursor]
                            * self.left_gain
                            + self.buffer.samples[self.cursor + 1] * self.right_gain;
                        self.cursor += 2;
                        samples_processed += 1;
                    }
                    (2, 2) => {
                        output[samples_processed] +=
                            self.buffer.samples[self.cursor] * self.left_gain;
                        output[samples_processed + 1] +=
                            self.buffer.samples[self.cursor + 1] * self.right_gain;
                        self.cursor += 2;
                        samples_processed += 2;
                    }
                    _ => break,
                }
            }

            if self.cursor >= self.buffer.sample_count() {
                if self.loop_mode {
                    self.cursor = 0;
                } else {
                    self.playing = false;
                }
            }
        }
    }

    fn recalculate_gains(&mut self) {
        let right_pan = self.pan * 0.5 + 0.5;
        self.right_gain = right_pan * self.gain;
        self.left_gain = (1.0 - right_pan) * self.gain;
    }
}

#[derive(Debug)]
pub struct SimpleMultiPlayer {
    channel_count: usize,
    sample_rate: u32,
    sources: Vec<OneShotSampleSource>,
    output_reset: bool,
}

impl SimpleMultiPlayer {
    pub fn new(channel_count: usize) -> Self {
        Self {
            channel_count: channel_count.max(1),
            sample_rate: 48_000,
            sources: Vec::new(),
            output_reset: false,
        }
    }

    pub fn add_sample_source(&mut self, source: OneShotSampleSource) -> usize {
        self.sources.push(source);
        self.sources.len() - 1
    }

    pub fn trigger_down(&mut self, index: usize) -> Result<()> {
        let source = self.sources.get_mut(index).ok_or(Error::InvalidArgument)?;
        source.trigger();
        Ok(())
    }

    pub fn trigger_up(&mut self, index: usize) -> Result<()> {
        let source = self.sources.get_mut(index).ok_or(Error::InvalidArgument)?;
        source.stop(true);
        Ok(())
    }

    pub fn reset_all(&mut self) {
        for source in &mut self.sources {
            source.stop(false);
        }
        self.output_reset = true;
    }

    pub fn render(&mut self, frame_count: usize) -> Vec<f32> {
        let mut output = vec![0.0; frame_count * self.channel_count];
        for source in &mut self.sources {
            source.mix_into(&mut output, self.channel_count, frame_count);
        }
        output
    }

    pub fn output_reset(&self) -> bool {
        self.output_reset
    }

    pub fn clear_output_reset(&mut self) {
        self.output_reset = false;
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot_sample_source_applies_center_pan() {
        let buffer = SampleBuffer::new(vec![1.0], 1, 48_000).unwrap();
        let mut source = OneShotSampleSource::new(buffer, 0.0);
        source.trigger();
        let mut output = vec![0.0; 2];
        source.mix_into(&mut output, 2, 1);
        assert_eq!(output, vec![0.5, 0.5]);
        assert!(!source.is_playing());
    }
}
