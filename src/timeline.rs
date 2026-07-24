use std::time::Duration;

/// The result of sending an input signal to the active flourish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalResult {
    /// No flourish was active.
    Ignored,
    /// The first signal started the flourish's graceful exit.
    ExitStarted,
    /// A signal during graceful exit requested immediate removal.
    HideImmediately,
}

/// A time-based lifecycle update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineUpdate {
    /// The flourish remains active.
    Active,
    /// The hold phase reached its ceiling and the graceful exit began on its
    /// own, without any input from the presenter.
    HoldExpired,
    /// Its graceful exit naturally completed.
    HideCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Idle,
    Holding {
        started_at: Duration,
    },
    Exiting {
        effect_started_at: Duration,
        exit_started_at: Duration,
    },
}

/// Renderer-independent state machine shared by every flourish.
///
/// The hold phase is deliberately bounded. A flourish owns the whole screen and
/// hides the cursor, and the only manual way out is input delivered to the
/// overlay window itself — which a presenter cannot always provide (the pointer
/// may be on a second display, or focus may sit elsewhere). The ceiling
/// guarantees the screen always comes back.
#[derive(Clone, Copy, Debug)]
pub struct Timeline {
    phase: Phase,
    exit_duration: Duration,
    hold_limit: Duration,
}

impl Timeline {
    #[must_use]
    pub const fn new(exit_duration: Duration, hold_limit: Duration) -> Self {
        Self {
            phase: Phase::Idle,
            exit_duration,
            hold_limit,
        }
    }

    /// An idle timeline that will never animate; useful before any flourish has
    /// been chosen.
    #[must_use]
    pub const fn idle() -> Self {
        Self::new(Duration::ZERO, Duration::ZERO)
    }

    #[must_use]
    pub const fn is_active(self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }

    /// Whether the flourish is still holding rather than exiting.
    #[must_use]
    pub const fn is_holding(self) -> bool {
        matches!(self.phase, Phase::Holding { .. })
    }

    pub const fn start(&mut self, now: Duration) {
        self.phase = Phase::Holding { started_at: now };
    }

    pub fn signal(&mut self, now: Duration) -> SignalResult {
        match self.phase {
            Phase::Idle => SignalResult::Ignored,
            Phase::Holding { started_at } => {
                self.phase = Phase::Exiting {
                    effect_started_at: started_at,
                    exit_started_at: now,
                };
                SignalResult::ExitStarted
            }
            Phase::Exiting { .. } => {
                self.phase = Phase::Idle;
                SignalResult::HideImmediately
            }
        }
    }

    pub const fn complete(&mut self) {
        self.phase = Phase::Idle;
    }

    pub fn update(&mut self, now: Duration) -> TimelineUpdate {
        match self.phase {
            Phase::Idle => TimelineUpdate::Active,
            Phase::Holding { started_at } => {
                if now.saturating_sub(started_at) >= self.hold_limit {
                    self.phase = Phase::Exiting {
                        effect_started_at: started_at,
                        exit_started_at: now,
                    };
                    return TimelineUpdate::HoldExpired;
                }
                TimelineUpdate::Active
            }
            Phase::Exiting {
                exit_started_at, ..
            } => {
                if now.saturating_sub(exit_started_at) >= self.exit_duration {
                    self.phase = Phase::Idle;
                    return TimelineUpdate::HideCompleted;
                }
                TimelineUpdate::Active
            }
        }
    }

    /// Normalized graceful-exit progress in the inclusive range `0..=1`.
    #[must_use]
    pub fn exit_progress(self, now: Duration) -> Option<f32> {
        let Phase::Exiting {
            exit_started_at, ..
        } = self.phase
        else {
            return None;
        };

        if self.exit_duration.is_zero() {
            return Some(1.0);
        }

        let elapsed = now.saturating_sub(exit_started_at).as_secs_f32();
        Some((elapsed / self.exit_duration.as_secs_f32()).clamp(0.0, 1.0))
    }

    /// Seconds since the current flourish started holding.
    #[must_use]
    pub fn effect_time(self, now: Duration) -> f32 {
        match self.phase {
            Phase::Idle => 0.0,
            Phase::Holding { started_at }
            | Phase::Exiting {
                effect_started_at: started_at,
                ..
            } => now.saturating_sub(started_at).as_secs_f32(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXIT: Duration = Duration::from_millis(1_800);
    const HOLD: Duration = Duration::from_secs(15);

    fn timeline() -> Timeline {
        Timeline::new(EXIT, HOLD)
    }

    #[test]
    fn first_signal_starts_a_graceful_exit() {
        let mut timeline = timeline();
        timeline.start(Duration::ZERO);

        assert_eq!(
            timeline.signal(Duration::from_secs(2)),
            SignalResult::ExitStarted
        );
        assert_eq!(timeline.exit_progress(Duration::from_secs(2)), Some(0.0));
        assert!(timeline.is_active());
    }

    #[test]
    fn second_signal_during_exit_hides_immediately() {
        let mut timeline = timeline();
        timeline.start(Duration::ZERO);
        timeline.signal(Duration::from_secs(1));

        assert_eq!(
            timeline.signal(Duration::from_millis(1_200)),
            SignalResult::HideImmediately
        );
        assert!(!timeline.is_active());
    }

    #[test]
    fn graceful_exit_completes_at_its_deadline() {
        let mut timeline = timeline();
        timeline.start(Duration::ZERO);
        timeline.signal(Duration::from_millis(200));

        assert_eq!(
            timeline.update(Duration::from_millis(1_999)),
            TimelineUpdate::Active
        );
        assert_eq!(
            timeline.update(Duration::from_secs(2)),
            TimelineUpdate::HideCompleted
        );
        assert!(!timeline.is_active());
    }

    #[test]
    fn natural_completion_returns_to_idle() {
        let mut timeline = timeline();
        timeline.start(Duration::ZERO);
        timeline.complete();

        assert!(!timeline.is_active());
        assert_eq!(timeline.signal(Duration::ZERO), SignalResult::Ignored);
    }

    #[test]
    fn exit_progress_is_clamped() {
        let mut timeline = timeline();
        timeline.start(Duration::ZERO);
        timeline.signal(Duration::from_secs(1));

        let halfway = timeline.exit_progress(Duration::from_millis(1_900));
        assert_eq!(halfway, Some(0.5));
        assert_eq!(timeline.exit_progress(Duration::from_secs(9)), Some(1.0));
    }

    #[test]
    fn effect_time_does_not_restart_when_exit_begins() {
        let mut timeline = timeline();
        timeline.start(Duration::from_secs(1));
        timeline.signal(Duration::from_secs(3));

        let elapsed = timeline.effect_time(Duration::from_secs(4));
        assert!((elapsed - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn an_unattended_hold_dismisses_itself() {
        // The presenter never clicks: the overlay must still give the screen
        // back on its own.
        let mut timeline = timeline();
        timeline.start(Duration::ZERO);

        let just_shy = HOLD
            .checked_sub(Duration::from_millis(1))
            .expect("the hold ceiling is longer than a millisecond");
        assert_eq!(timeline.update(just_shy), TimelineUpdate::Active);
        assert!(timeline.is_holding());

        assert_eq!(timeline.update(HOLD), TimelineUpdate::HoldExpired);
        assert!(!timeline.is_holding());
        assert!(timeline.is_active());
        assert_eq!(timeline.exit_progress(HOLD), Some(0.0));

        assert_eq!(timeline.update(HOLD + EXIT), TimelineUpdate::HideCompleted);
        assert!(!timeline.is_active());
    }

    #[test]
    fn the_hold_ceiling_preserves_the_effect_clock() {
        let mut timeline = timeline();
        timeline.start(Duration::from_secs(2));
        timeline.update(Duration::from_secs(2) + HOLD);

        // The exit animates from the flourish's own elapsed time, not from zero.
        let elapsed = timeline.effect_time(Duration::from_secs(2) + HOLD);
        assert!((elapsed - HOLD.as_secs_f32()).abs() < 0.001);
    }

    #[test]
    fn a_manual_signal_still_wins_before_the_ceiling() {
        let mut timeline = timeline();
        timeline.start(Duration::ZERO);
        assert_eq!(
            timeline.signal(Duration::from_secs(1)),
            SignalResult::ExitStarted
        );

        // Passing the hold ceiling mid-exit must not restart the exit.
        assert_eq!(timeline.update(HOLD), TimelineUpdate::HideCompleted);
    }
}
