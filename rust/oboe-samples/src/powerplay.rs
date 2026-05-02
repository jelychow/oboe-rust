use crate::iolib::{OneShotSampleSource, SampleBuffer, SimpleMultiPlayer};
use oboe_core::error::{Error, Result};

pub use oboe_core::types::PerformanceMode;

#[derive(Debug)]
pub struct PowerPlayPlayer {
    player: SimpleMultiPlayer,
    tracks: Vec<Option<usize>>,
    current_track: Option<usize>,
    performance_mode: PerformanceMode,
    mmap_enabled: bool,
    buffer_size_in_frames: i32,
    buffer_capacity_in_frames: i32,
    playback_position_frames: usize,
}

impl PowerPlayPlayer {
    pub fn new(channel_count: usize) -> Self {
        Self {
            player: SimpleMultiPlayer::new(channel_count),
            tracks: Vec::new(),
            current_track: None,
            performance_mode: PerformanceMode::None,
            mmap_enabled: false,
            buffer_size_in_frames: 0,
            buffer_capacity_in_frames: 192,
            playback_position_frames: 0,
        }
    }

    pub fn load_track(
        &mut self,
        track_index: usize,
        samples: Vec<f32>,
        sample_channels: usize,
    ) -> Result<()> {
        let buffer = SampleBuffer::new(samples, sample_channels, 48_000)?;
        let source_index = self
            .player
            .add_sample_source(OneShotSampleSource::new(buffer, 0.0));
        if self.tracks.len() <= track_index {
            self.tracks.resize(track_index + 1, None);
        }
        self.tracks[track_index] = Some(source_index);
        Ok(())
    }

    pub fn start_playing(&mut self, track_index: usize, mode: PerformanceMode) -> Result<()> {
        let source_index = self
            .tracks
            .get(track_index)
            .and_then(|entry| *entry)
            .ok_or(Error::InvalidArgument)?;
        self.performance_mode = mode;
        self.current_track = Some(track_index);
        self.playback_position_frames = 0;
        self.player.trigger_down(source_index)
    }

    pub fn stop_playing(&mut self, track_index: usize) -> Result<()> {
        let source_index = self
            .tracks
            .get(track_index)
            .and_then(|entry| *entry)
            .ok_or(Error::InvalidArgument)?;
        if self.current_track == Some(track_index) {
            self.current_track = None;
        }
        self.player.trigger_up(source_index)
    }

    pub fn update_performance_mode(&mut self, mode: PerformanceMode) {
        self.performance_mode = mode;
    }

    pub fn performance_mode(&self) -> PerformanceMode {
        self.performance_mode
    }

    pub fn set_mmap_enabled(&mut self, enabled: bool) -> bool {
        self.mmap_enabled = enabled;
        self.mmap_enabled
    }

    pub fn mmap_enabled(&self) -> bool {
        self.mmap_enabled
    }

    pub fn currently_playing_index(&self) -> Option<usize> {
        self.current_track
    }

    pub fn set_buffer_size_in_frames(&mut self, buffer_size_in_frames: i32) -> i32 {
        self.buffer_size_in_frames = buffer_size_in_frames.clamp(0, self.buffer_capacity_in_frames);
        self.buffer_size_in_frames
    }

    pub fn buffer_capacity_in_frames(&self) -> i32 {
        self.buffer_capacity_in_frames
    }

    pub fn playback_position_millis(&self) -> i64 {
        (self.playback_position_frames as i64 * 1_000) / self.player.sample_rate() as i64
    }

    pub fn seek_to(&mut self, position_millis: i32) {
        self.playback_position_frames =
            (position_millis.max(0) as usize * self.player.sample_rate() as usize) / 1_000;
    }

    pub fn render(&mut self, frame_count: usize) -> Vec<f32> {
        self.playback_position_frames += frame_count;
        self.player.render(frame_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powerplay_tracks_current_index_and_mode() {
        let mut player = PowerPlayPlayer::new(1);
        player.load_track(2, vec![1.0], 1).unwrap();
        player
            .start_playing(2, PerformanceMode::PowerSaving)
            .unwrap();
        assert_eq!(player.currently_playing_index(), Some(2));
        assert_eq!(player.performance_mode(), PerformanceMode::PowerSaving);
    }
}
