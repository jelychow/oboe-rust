use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapResult {
    Early,
    Good,
    Late,
    Miss,
}

#[derive(Debug)]
pub struct RhythmGame {
    clap_windows: VecDeque<i64>,
    window_center_offset_ms: i64,
}

impl RhythmGame {
    pub fn new(clap_windows: Vec<i64>, window_center_offset_ms: i64) -> Self {
        Self {
            clap_windows: clap_windows.into(),
            window_center_offset_ms,
        }
    }

    pub fn tap(&mut self, tap_time_millis: i64) -> TapResult {
        let Some(window) = self.clap_windows.pop_front() else {
            return TapResult::Miss;
        };
        if tap_time_millis < window - self.window_center_offset_ms {
            TapResult::Early
        } else if tap_time_millis <= window + self.window_center_offset_ms {
            TapResult::Good
        } else {
            TapResult::Late
        }
    }

    pub fn pending_windows(&self) -> usize {
        self.clap_windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rhythm_game_consumes_one_window_per_tap() {
        let mut game = RhythmGame::new(vec![100], 20);
        assert_eq!(game.tap(150), TapResult::Late);
        assert_eq!(game.tap(100), TapResult::Miss);
    }
}
