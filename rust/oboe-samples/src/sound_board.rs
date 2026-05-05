use crate::shared::{mix_mono_tracks, mono_to_stereo, SynthSound};

const BASE_FREQUENCY: f32 = 196.0;
const FREQUENCY_MULTIPLIER: f32 = 1.059_463_1;
const BASE_AMPLITUDE: f32 = 0.20;
const AMPLITUDE_MULTIPLIER: f32 = 0.96;

#[derive(Debug)]
pub struct SoundBoardSynth {
    notes: Vec<SynthSound>,
    channel_count: usize,
}

impl SoundBoardSynth {
    pub fn new(sample_rate: i32, channel_count: usize, note_count: usize) -> Self {
        let mut frequency = BASE_FREQUENCY;
        let mut amplitude = BASE_AMPLITUDE;
        let mut notes = Vec::with_capacity(note_count);
        for _ in 0..note_count {
            notes.push(SynthSound::new(sample_rate, frequency, amplitude));
            frequency *= FREQUENCY_MULTIPLIER;
            amplitude *= AMPLITUDE_MULTIPLIER;
        }

        Self {
            notes,
            channel_count: channel_count.max(1),
        }
    }

    pub fn note_on(&mut self, note_index: usize) {
        if let Some(note) = self.notes.get_mut(note_index) {
            note.note_on();
        }
    }

    pub fn note_off(&mut self, note_index: usize) {
        if let Some(note) = self.notes.get_mut(note_index) {
            note.note_off();
        }
    }

    pub fn tap(&mut self, is_on: bool) {
        for note in &mut self.notes {
            if is_on {
                note.note_on();
            } else {
                note.note_off();
            }
        }
    }

    pub fn render(&mut self, frame_count: usize) -> Vec<f32> {
        let mut tracks = Vec::with_capacity(self.notes.len());
        for note in &mut self.notes {
            let mut mono = vec![0.0; frame_count];
            note.render_mono(&mut mono);
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
    fn sound_board_ignores_out_of_range_notes() {
        let mut synth = SoundBoardSynth::new(48_000, 1, 1);
        synth.note_on(99);
        assert_eq!(synth.render(1), vec![0.0]);
    }
}
