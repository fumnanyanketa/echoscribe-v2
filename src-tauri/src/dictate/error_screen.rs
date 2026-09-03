//! Which of the small dark window's three screens an error kind belongs on.
//!
//! Record 0002's clearing table has two readers and, until now, had two copies:
//! `src/main.js` decided which screen was drawn, and `src/shell/mod.rs` decided
//! whether the small window was on screen at all. Neither reader could be
//! deleted, so the table stopped being copied and started being handed over
//! (record 0004's second amendment of 2026-09-03, and record 0002's fifteenth).
//!
//! This file is that one copy. It lives in the dictate feature because this is
//! where the kinds are minted, in `microphone.rs` and `deepgram_key.rs`. Rust
//! classifies a kind once, `dictation:error` carries the answer in its `screen`
//! field, and the interface mounts the screen it is named and inspects no kind.
//! Record 0004's shell reads the same answer through `screen_for` below.
//!
//! What is left over is each family's pairing with its own proof, which stays on
//! both sides because each side clears the thing it is itself holding. That
//! residue is held together by source guards: `the_three_families_each_clear_on`
//! `_their_own_proof` in `mod.rs` for the interface side, and
//! `each_proof_clears_only_its_own_screen` in `shell/mod.rs` for the Rust side.

/// One of the three screens the 760x540 dark window can hold for an error.
/// There is no fourth: everything else that window shows is sign in or first
/// run, and neither is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorScreen {
    /// The microphone would not open, or died mid dictation (record 0002 AC-28,
    /// AC-29). Proved wrong by the microphone opening.
    Microphone,
    /// A key stopped being accepted, or none is saved. Proved wrong by a key
    /// being accepted and saved.
    KeySetup,
    /// Any other mid dictation Deepgram ending (AC-30). Proved wrong by the
    /// first finalised words coming back.
    Deepgram,
}

impl ErrorScreen {
    /// The name the interface knows this screen by, which is the same name
    /// `src/main.js` records as the mounted screen. It crosses on the event.
    pub fn name(self) -> &'static str {
        match self {
            ErrorScreen::Microphone => "mic-error",
            ErrorScreen::KeySetup => "key-setup",
            ErrorScreen::Deepgram => "deepgram-error",
        }
    }
}

/// Classify an error kind into its screen, or `None` when it has none.
///
/// Every kind this feature mints is placed here, and a kind that is not is
/// refused loudly on stderr rather than passed on quietly: the interface can no
/// longer look at a kind, so an unplaced one reaches no screen at all and a
/// person would be told nothing about a failure that did happen. Record 0002's
/// fifteenth amendment names that cost and requires this refusal.
///
/// `every_kind_this_feature_mints_is_placed_on_a_screen` below reads
/// `microphone.rs` and `deepgram_key.rs` and fails the build when a new kind is
/// minted without being placed, so the refusal is normally caught long before
/// anybody's microphone is involved.
pub fn screen_for(kind: &str) -> Option<ErrorScreen> {
    match kind {
        // The four microphone kinds, whether the microphone would not open or
        // stopped working mid dictation. Record 0002 fixes them at four and
        // never a fifth, and the mid dictation case reuses the same kind with a
        // second sentence rather than adding one.
        "microphone_blocked_by_windows"
        | "microphone_in_use_by_another_app"
        | "no_microphone_found"
        | "microphone_unavailable" => Some(ErrorScreen::Microphone),

        // A saved key that stopped being accepted, and a key that is not there
        // at all, both need the setup screen, because that is the only place a
        // new one can be pasted (record 0002 AC-9, AC-13's Replace key).
        // `no_deepgram_key` never rides on `dictation:error`, which sends
        // `dictation:needs_key` instead; it reaches here off a failed
        // `retry_dictation`, whose payload carries the same named kinds.
        "deepgram_key_rejected" | "no_deepgram_key" => Some(ErrorScreen::KeySetup),

        // Every other mid dictation Deepgram ending. `deepgram_unreachable` and
        // `deepgram_check_failed` are on this screen when they arrive as an
        // ending; the clearing table's middle row is about the other place they
        // appear, drawn inline on the setup screen by the key check itself.
        "deepgram_no_allowance"
        | "deepgram_unreachable"
        | "deepgram_check_failed"
        | "deepgram_key_not_allowed"
        | "deepgram_connection_lost" => Some(ErrorScreen::Deepgram),

        // Two kinds with deliberately no screen on this window, and they are
        // not omissions. The password refusal rides on `dictation:blocked` to
        // the pill alone and the window stays exactly where it is (AC-20).
        // `not_signed_in` is every command's refusal, and a person who is not
        // signed in has one screen, which is sign in (record 0003 AC-1).
        "blocked_password_field" | "not_signed_in" => None,

        other => {
            eprintln!(
                "dictate: the error kind `{other}` is not placed on a screen, so nobody will be \
                 told about it. Place it in dictate/error_screen.rs (record 0002, fifteenth \
                 amendment)"
            );
            None
        }
    }
}

/// The screen's name for an error kind, ready to ride on an event or a command
/// error beside the kind itself. `None` for a kind with no screen, which
/// crosses as `null` and mounts nothing.
pub fn screen_name_for(kind: &str) -> Option<&'static str> {
    screen_for(kind).map(ErrorScreen::name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every kind literal in one file's `kind` method, which is the one place
    /// each error type names the strings that cross to the interface.
    fn kinds_minted_in(source: &str) -> Vec<String> {
        let start = source
            .find("pub fn kind(self) -> &'static str {")
            .expect("both error types name their kinds in a `kind` method");
        let body = &source[start..];
        let end = body
            .find("\n    }")
            .expect("that method has a closing brace");
        body[..end]
            .split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect()
    }

    fn every_minted_kind() -> Vec<String> {
        let mut all = kinds_minted_in(include_str!("microphone.rs"));
        all.extend(kinds_minted_in(include_str!("deepgram_key.rs")));
        all
    }

    #[test]
    fn every_kind_this_feature_mints_is_placed_on_a_screen() {
        // covers: record 0002 AC-13, AC-28, AC-30, and the fifteenth
        // amendment's requirement that the classifier place every kind it is
        // given or refuse loudly. The interface can no longer look at a kind,
        // so a kind nobody placed reaches no screen and a person is told
        // nothing about a failure that did happen. This is that refusal, moved
        // to the build.
        let minted = every_minted_kind();
        assert!(
            minted.len() >= 10,
            "only {} kinds were read out of microphone.rs and deepgram_key.rs, so this guard has \
             stopped reading them and is no longer checking anything",
            minted.len()
        );
        for kind in minted {
            assert!(
                screen_for(&kind).is_some(),
                "the error kind `{kind}` is minted but is not placed on a screen in \
                 error_screen.rs. The interface inspects no kind, so it would reach nobody \
                 (record 0002, fifteenth amendment)"
            );
        }
    }

    #[test]
    fn the_four_microphone_kinds_are_the_microphone_screen() {
        // covers: record 0002 AC-28, AC-29. These are the kinds whose screen
        // clears on the microphone opening, and they are four, never a fifth.
        for kind in kinds_minted_in(include_str!("microphone.rs")) {
            assert_eq!(screen_for(&kind), Some(ErrorScreen::Microphone), "{kind}");
        }
    }

    #[test]
    fn a_rejected_key_is_the_setup_screen_and_not_the_deepgram_one() {
        // covers: record 0002 AC-13. Replace key opens the setup screen, where
        // a new key can actually be pasted, and that screen clears on a key
        // being accepted rather than on words coming back.
        assert_eq!(
            screen_for("deepgram_key_rejected"),
            Some(ErrorScreen::KeySetup)
        );
        assert_eq!(screen_for("no_deepgram_key"), Some(ErrorScreen::KeySetup));
    }

    #[test]
    fn the_other_deepgram_endings_are_the_deepgram_screen() {
        // covers: record 0002 AC-30.
        for kind in [
            "deepgram_no_allowance",
            "deepgram_unreachable",
            "deepgram_check_failed",
            "deepgram_key_not_allowed",
            "deepgram_connection_lost",
        ] {
            assert_eq!(screen_for(kind), Some(ErrorScreen::Deepgram), "{kind}");
        }
    }

    #[test]
    fn a_kind_with_no_screen_is_not_given_one() {
        // covers: record 0002 AC-20, and record 0003 AC-1. The password
        // refusal goes to the pill alone and the window stays where it is, and
        // a person who is not signed in has exactly one screen.
        assert_eq!(screen_for("blocked_password_field"), None);
        assert_eq!(screen_for("not_signed_in"), None);
        assert_eq!(screen_for(""), None);
        assert_eq!(screen_for("something_nobody_placed"), None);
    }

    #[test]
    fn the_interface_names_no_error_kind_of_its_own() {
        // covers: record 0002's fifteenth amendment, as a source guard only.
        // The whole point of the handover is that the table has one copy. A
        // kind literal reappearing in the shell is the second copy growing
        // back, and it drifts silently because both copies keep working until
        // they disagree.
        const INTERFACE_SHELL: &str = include_str!("../../../src/main.js");
        for kind in every_minted_kind() {
            assert!(
                !INTERFACE_SHELL.contains(&kind),
                "src/main.js names the error kind `{kind}` again. Rust classifies a kind into \
                 its screen once and the event carries the answer; the interface mounts what it \
                 is named and inspects no kind (record 0002, fifteenth amendment)"
            );
        }
    }
}
