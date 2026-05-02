use crate::iolib::{OneShotSampleSource, SampleBuffer, SimpleMultiPlayer};
use oboe_core::error::{Error, Result};

#[derive(Debug)]
pub struct DrumThumper {
    player: SimpleMultiPlayer,
    pad_to_source: Vec<Option<usize>>,
}

impl DrumThumper {
    pub fn new(channel_count: usize) -> Self {
        Self {
            player: SimpleMultiPlayer::new(channel_count),
            pad_to_source: Vec::new(),
        }
    }

    pub fn load_pad(
        &mut self,
        pad_index: usize,
        samples: Vec<f32>,
        sample_channels: usize,
        pan: f32,
    ) -> Result<()> {
        let buffer = SampleBuffer::new(samples, sample_channels, 48_000)?;
        let source_index = self
            .player
            .add_sample_source(OneShotSampleSource::new(buffer, pan));
        if self.pad_to_source.len() <= pad_index {
            self.pad_to_source.resize(pad_index + 1, None);
        }
        self.pad_to_source[pad_index] = Some(source_index);
        Ok(())
    }

    pub fn trigger(&mut self, pad_index: usize) -> Result<()> {
        let source_index = self
            .pad_to_source
            .get(pad_index)
            .and_then(|entry| *entry)
            .ok_or(Error::InvalidArgument)?;
        self.player.trigger_down(source_index)
    }

    pub fn stop_trigger(&mut self, pad_index: usize) -> Result<()> {
        let source_index = self
            .pad_to_source
            .get(pad_index)
            .and_then(|entry| *entry)
            .ok_or(Error::InvalidArgument)?;
        self.player.trigger_up(source_index)
    }

    pub fn render(&mut self, frame_count: usize) -> Vec<f32> {
        self.player.render(frame_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drum_pad_trigger_maps_to_loaded_source() {
        let mut drums = DrumThumper::new(1);
        drums.load_pad(3, vec![0.75], 1, 0.0).unwrap();
        assert_eq!(drums.trigger(2), Err(Error::InvalidArgument));
        drums.trigger(3).unwrap();
        assert_eq!(drums.render(1), vec![0.75]);
    }
}
