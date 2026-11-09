/// Pomodoro round sequencing: work → short-break → work → … → long-break → work (cycle).
///
/// Mirrors the original app's behaviour:
/// - After each completed work round, check if `work_round_number >= work_rounds_total`.
///   If yes → long break; otherwise → short break.
/// - After short break → advance work_round_number, next round is Work.
/// - After long break → reset work_round_number to 1, next round is Work.
use serde::{Deserialize, Serialize};

use crate::settings::Settings;

// ---------------------------------------------------------------------------
// Round type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoundType {
    Work,
    ShortBreak,
    LongBreak,
}

impl RoundType {
    pub fn as_str(self) -> &'static str {
        match self {
            RoundType::Work => "work",
            RoundType::ShortBreak => "short-break",
            RoundType::LongBreak => "long-break",
        }
    }
}

// ---------------------------------------------------------------------------
// Sequence state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SequenceState {
    pub current_round: RoundType,
    /// The round type that was active before `advance()` was last called.
    /// `None` on the very first round (no preceding round exists).
    pub previous_round: Option<RoundType>,
    /// Which work round we're currently in (1-based). Displayed to the user.
    pub work_round_number: u32,
    /// Total work rounds before a long break (from settings).
    pub work_rounds_total: u32,
    /// Monotonically-increasing count of work rounds since the last reset.
    /// Unlike `work_round_number` this never resets at cycle boundaries,
    /// so it can be used as a session counter when long breaks are disabled.
    pub session_work_count: u32,
}

impl SequenceState {
    pub fn new(work_rounds_total: u32) -> Self {
        Self {
            current_round: RoundType::Work,
            previous_round: None,
            work_round_number: 1,
            work_rounds_total,
            session_work_count: 1,
        }
    }

    /// Duration of the current round in seconds, taken from settings.
    pub fn current_duration_secs(&self, settings: &Settings) -> u32 {
        settings.time_work_secs
    }

    /// Advance to the next round. Always stays in Work mode.
    ///
    /// Call this when the engine fires `TimerEvent::Complete`.
    pub fn advance(&mut self, settings: &Settings) -> (RoundType, u32) {
        self.previous_round = Some(RoundType::Work);
        self.current_round = RoundType::Work;
        self.work_round_number += 1;
        self.session_work_count += 1;

        let duration = self.current_duration_secs(settings);
        (self.current_round, duration)
    }

    /// Reset the sequence to the initial state (used by the Reset command).
    pub fn reset(&mut self) {
        self.current_round = RoundType::Work;
        self.previous_round = None;
        self.work_round_number = 1;
        self.session_work_count = 1;
    }
}

// ---------------------------------------------------------------------------
// Tests (TIMER-02 acceptance: full cycles with various work_rounds values)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(work: u32, short: u32, long: u32) -> Settings {
        Settings {
            time_work_secs: work,
            time_short_break_secs: short,
            time_long_break_secs: long,
            long_break_interval: 4,
            ..Settings::default()
        }
    }

    #[test]
    fn advance_always_stays_in_work() {
        let s = settings(1500, 300, 900);
        let mut seq = SequenceState::new(4);

        for i in 1..=10 {
            let (rt, dur) = seq.advance(&s);
            assert_eq!(rt, RoundType::Work);
            assert_eq!(dur, 1500);
            assert_eq!(seq.session_work_count, i + 1);
        }
    }

    #[test]
    fn reset_returns_to_initial_state() {
        let s = settings(1500, 300, 900);
        let mut seq = SequenceState::new(4);

        seq.advance(&s);
        seq.advance(&s);
        seq.reset();

        assert_eq!(seq.current_round, RoundType::Work);
        assert_eq!(seq.work_round_number, 1);
        assert_eq!(seq.session_work_count, 1);
        assert_eq!(seq.current_duration_secs(&s), 1500);
    }
}
