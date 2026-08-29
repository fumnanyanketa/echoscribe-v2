//! The double-tap state machine.
//!
//! This is the whole of what the keyboard hook understands. It sees a stream of
//! key events, each already reduced to one of two classes by the caller:
//!
//!   * `Class::Modifier` - the chosen modifier, either the left or the right
//!     one (record 0002: "either side of the keyboard counts").
//!   * `Class::Other`    - literally anything else.
//!
//! It never learns which "other" key was pressed, and it stores none of them.
//! The only state it keeps is a small `Phase` value and, in two of the phases,
//! one `Instant`. There is no key buffer, no history, no vector. That is the
//! property the whole risk section of record 0002 rests on, so the tests below
//! assert the sequence rules and `Phase` stays this small on purpose.
//!
//! A "tap" is the chosen modifier going down and coming back up with no other
//! key pressed in between. Two taps count as a double tap when the second tap's
//! *press* lands within `WINDOW` of the first tap's *release* (record 0002:
//! measured release-to-press, so holding either key down never eats the
//! window).

use std::time::{Duration, Instant};

/// The gap allowed between the first tap's release and the second tap's press.
/// Fixed by record 0002, not a setting.
pub const WINDOW: Duration = Duration::from_millis(300);

/// Every key event, already reduced to what the machine cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// The chosen modifier (left or right both map here).
    Modifier,
    /// Any other key. The machine is told nothing more than this.
    Other,
}

/// Where the machine is in recognising a double tap.
///
/// `Contaminated` is the state after some other key joined the current hold: the
/// modifier is still down, but this tap can no longer count. It clears when the
/// modifier is released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Nothing in progress.
    Idle,
    /// The modifier is held for what could be the first tap, still clean.
    FirstHeld,
    /// Some other key joined the first hold; waiting for the modifier release
    /// that ends this dead tap.
    Contaminated,
    /// The first tap completed cleanly at `released`. Waiting for a second press.
    FirstDone { released: Instant },
    /// The modifier is held for the second tap, which began within the window
    /// at `first_released`, still clean.
    SecondHeld { first_released: Instant },
    /// Some other key joined the second hold; waiting for the release that ends
    /// it. The double tap is already lost.
    SecondContaminated,
}

/// Feeds on classified key events and reports when a double tap completes.
pub struct Machine {
    phase: Phase,
}

impl Default for Machine {
    fn default() -> Self {
        Self { phase: Phase::Idle }
    }
}

impl Machine {
    /// Handle one key event. `now` is the moment it happened; the caller passes
    /// `Instant::now()` and the tests pass a controlled clock.
    ///
    /// Returns `true` exactly once, on the key-up that completes a valid double
    /// tap. The caller turns that into "toggle dictation".
    pub fn on_key(&mut self, class: Class, down: bool, now: Instant) -> bool {
        match (self.phase, class, down) {
            // ---- from Idle -------------------------------------------------
            (Phase::Idle, Class::Modifier, true) => {
                self.phase = Phase::FirstHeld;
                false
            }
            (Phase::Idle, _, _) => false,

            // ---- first hold ---------------------------------------------------
            // The modifier auto-repeats while held; ignore the repeat downs.
            (Phase::FirstHeld, Class::Modifier, true) => false,
            (Phase::FirstHeld, Class::Modifier, false) => {
                self.phase = Phase::FirstDone { released: now };
                false
            }
            (Phase::FirstHeld, Class::Other, true) => {
                self.phase = Phase::Contaminated;
                false
            }
            (Phase::FirstHeld, Class::Other, false) => false,

            // ---- contaminated first hold -----------------------------------
            (Phase::Contaminated, Class::Modifier, false) => {
                self.phase = Phase::Idle;
                false
            }
            (Phase::Contaminated, _, _) => false,

            // ---- waiting for the second press ----------------------------------
            (Phase::FirstDone { released }, Class::Modifier, true) => {
                if now.duration_since(released) <= WINDOW {
                    self.phase = Phase::SecondHeld {
                        first_released: released,
                    };
                } else {
                    // Too slow. This press is the start of a fresh first tap.
                    self.phase = Phase::FirstHeld;
                }
                false
            }
            (Phase::FirstDone { .. }, Class::Modifier, false) => false,
            (Phase::FirstDone { .. }, Class::Other, true) => {
                self.phase = Phase::Idle;
                false
            }
            (Phase::FirstDone { .. }, Class::Other, false) => false,

            // ---- second hold ------------------------------------------------
            (Phase::SecondHeld { .. }, Class::Modifier, true) => false,
            (Phase::SecondHeld { .. }, Class::Modifier, false) => {
                // A clean second tap: down and up with nothing in between, and
                // the press was already inside the window. Fire.
                self.phase = Phase::Idle;
                true
            }
            (Phase::SecondHeld { .. }, Class::Other, true) => {
                self.phase = Phase::SecondContaminated;
                false
            }
            (Phase::SecondHeld { .. }, Class::Other, false) => false,

            // ---- contaminated second hold ---------------------------------
            (Phase::SecondContaminated, Class::Modifier, false) => {
                self.phase = Phase::Idle;
                false
            }
            (Phase::SecondContaminated, _, _) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clock that only moves when a test tells it to.
    struct Clock(Instant);
    impl Clock {
        fn new() -> Self {
            Self(Instant::now())
        }
        fn now(&self) -> Instant {
            self.0
        }
        fn advance(&mut self, by: Duration) {
            self.0 += by;
        }
    }

    /// Press then release the chosen modifier, cleanly.
    fn tap(m: &mut Machine, clock: &Clock) -> bool {
        m.on_key(Class::Modifier, true, clock.now());
        m.on_key(Class::Modifier, false, clock.now())
    }

    #[test]
    fn a_clean_double_tap_inside_the_window_fires_once() {
        let mut m = Machine::default();
        let mut clock = Clock::new();

        assert!(!tap(&mut m, &clock), "first tap must not fire");
        clock.advance(Duration::from_millis(150));
        assert!(tap(&mut m, &clock), "second tap inside the window fires");

        // And only once: the next tap is a fresh first tap.
        clock.advance(Duration::from_millis(10));
        assert!(!tap(&mut m, &clock));
    }

    #[test]
    fn the_window_is_measured_from_release_to_press_not_press_to_press() {
        let mut m = Machine::default();
        let mut clock = Clock::new();

        // Hold the first tap down for a long time. Under a press-to-press
        // reading this would blow the budget; under release-to-press it must
        // not, because the hold is not counted.
        m.on_key(Class::Modifier, true, clock.now());
        clock.advance(Duration::from_millis(900));
        assert!(!m.on_key(Class::Modifier, false, clock.now()));

        clock.advance(Duration::from_millis(250)); // gap: within 300ms
        assert!(
            tap(&mut m, &clock),
            "gap was inside the window, so it fires"
        );
    }

    #[test]
    fn a_second_press_after_the_window_does_not_fire() {
        let mut m = Machine::default();
        let mut clock = Clock::new();

        assert!(!tap(&mut m, &clock));
        clock.advance(Duration::from_millis(301));
        assert!(!tap(&mut m, &clock), "just outside the window: no fire");

        // That late press became a fresh first tap, so a prompt follow-up fires.
        clock.advance(Duration::from_millis(50));
        assert!(tap(&mut m, &clock));
    }

    #[test]
    fn ctrl_c_then_ctrl_v_never_fires() {
        // AC-6: two modifier presses with another key in between must not start
        // dictation. Sequence: mod down, C down, C up, mod up, mod down, V down,
        // V up, mod up.
        let mut m = Machine::default();
        let clock = Clock::new();

        assert!(!m.on_key(Class::Modifier, true, clock.now()));
        assert!(!m.on_key(Class::Other, true, clock.now())); // C
        assert!(!m.on_key(Class::Other, false, clock.now()));
        assert!(!m.on_key(Class::Modifier, false, clock.now())); // tap 1 void

        assert!(!m.on_key(Class::Modifier, true, clock.now()));
        assert!(!m.on_key(Class::Other, true, clock.now())); // V
        assert!(!m.on_key(Class::Other, false, clock.now()));
        assert!(!m.on_key(Class::Modifier, false, clock.now())); // tap 2 void
    }

    #[test]
    fn an_other_key_during_the_gap_resets_the_sequence() {
        let mut m = Machine::default();
        let clock = Clock::new();

        assert!(!tap(&mut m, &clock)); // clean first tap, now in the gap
        assert!(!m.on_key(Class::Other, true, clock.now())); // types a letter
        assert!(!m.on_key(Class::Other, false, clock.now()));

        // The next modifier press is a fresh first tap, not a second one.
        assert!(!tap(&mut m, &clock));
        assert!(tap(&mut m, &clock), "now the pair completes");
    }

    #[test]
    fn a_contaminated_second_tap_does_not_fire() {
        let mut m = Machine::default();
        let clock = Clock::new();

        assert!(!tap(&mut m, &clock)); // first tap clean
        assert!(!m.on_key(Class::Modifier, true, clock.now())); // second press, in window
        assert!(!m.on_key(Class::Other, true, clock.now())); // but another key joins
        assert!(!m.on_key(Class::Other, false, clock.now()));
        assert!(
            !m.on_key(Class::Modifier, false, clock.now()),
            "second tap was dirty"
        );
    }

    #[test]
    fn modifier_auto_repeat_while_held_is_ignored() {
        let mut m = Machine::default();
        let clock = Clock::new();

        m.on_key(Class::Modifier, true, clock.now());
        for _ in 0..20 {
            assert!(!m.on_key(Class::Modifier, true, clock.now()), "repeat down");
        }
        assert!(!m.on_key(Class::Modifier, false, clock.now())); // first tap done

        assert!(
            tap(&mut m, &clock),
            "double tap still recognised after repeats"
        );
    }

    #[test]
    fn three_taps_fire_once_then_leave_a_fresh_first_tap() {
        let mut m = Machine::default();
        let mut clock = Clock::new();

        assert!(!tap(&mut m, &clock));
        clock.advance(Duration::from_millis(100));
        assert!(tap(&mut m, &clock), "taps one and two fire");
        clock.advance(Duration::from_millis(100));
        assert!(!tap(&mut m, &clock), "tap three is just a fresh first tap");
        clock.advance(Duration::from_millis(100));
        assert!(tap(&mut m, &clock), "tap four completes the new pair");
    }
}
