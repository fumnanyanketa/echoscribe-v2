//! What the nav rail holds, and where the dashboard lands.
//!
//! A fixed list, in Rust, handed to the interface as identifiers. Not a list
//! the interface holds, for the same reason `get_hotkey` hands out the two
//! hotkeys rather than letting a screen offer a third: a screen must not be
//! able to invent a destination. That is record 0004's AC-2 as a rule in Rust
//! and not only a rule in the design.
//!
//! It returns no wording. `design/registry.md` owns what a person reads, and
//! the interface reads it from there, exactly as it does for every other screen
//! in this app.
//!
//! The list holds only sections that work. Today that is Settings, with
//! Dictation, Languages, Vocabulary and Transcription beneath it, in the
//! comp's order. History is not here, not even disabled: `design/registry.md`
//! draws `Nav item` with exactly two states, so a disabled third does not
//! exist in this design system, and a nav item that opens nothing is a promise
//! the app cannot keep. Each of plan rows 3, 4 and 5 adds its own item by an
//! amendment to record 0004 as it lands, and **Languages and Vocabulary joined
//! on 2026-09-04**, by the third amendment of that day, for records 0006 and
//! 0005. Plan row 5's History is the one still to come, and it moves the
//! landing destination when it does.

use serde::Serialize;

/// One entry in the rail. `children` is the `Section sub-nav` beneath it, and
/// is empty for an item that has none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RailItem {
    pub id: &'static str,
    pub children: Vec<RailItem>,
}

/// The rail, and which of its destinations the dashboard opens on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RailView {
    pub items: Vec<RailItem>,
    /// The landing destination. Settings, on its Dictation sub-section, and
    /// temporary by design: it is the only section that exists, so it is the
    /// only honest landing. Plan row 5's record moves it to History and amends
    /// record 0004 when it is designed.
    pub landing: &'static str,
}

/// Every identifier the rail can ever hand out today. One place, so a reader
/// can see the whole list at once and so the tests below cannot drift from it.
pub const SETTINGS: &str = "settings";
pub const SETTINGS_DICTATION: &str = "settings.dictation";
pub const SETTINGS_LANGUAGES: &str = "settings.languages";
pub const SETTINGS_VOCABULARY: &str = "settings.vocabulary";
pub const SETTINGS_TRANSCRIPTION: &str = "settings.transcription";

/// The rail as it stands. A literal, so "the rail being unreadable" is not a
/// failure that can happen (record 0004, Interface surface).
pub fn view() -> RailView {
    RailView {
        items: vec![RailItem {
            id: SETTINGS,
            // The comp's order, which `design/registry.md`'s
            // `Section sub-nav` names: Dictation, Languages, Vocabulary,
            // Transcription. The two new ones go between the two that existed
            // rather than after them.
            children: vec![
                RailItem {
                    id: SETTINGS_DICTATION,
                    children: Vec::new(),
                },
                RailItem {
                    id: SETTINGS_LANGUAGES,
                    children: Vec::new(),
                },
                RailItem {
                    id: SETTINGS_VOCABULARY,
                    children: Vec::new(),
                },
                RailItem {
                    id: SETTINGS_TRANSCRIPTION,
                    children: Vec::new(),
                },
            ],
        }],
        landing: SETTINGS_DICTATION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every destination in the rail, parents and children alike, flattened.
    fn destinations(items: &[RailItem]) -> Vec<&'static str> {
        let mut out = Vec::new();
        for item in items {
            out.push(item.id);
            out.extend(destinations(&item.children));
        }
        out
    }

    #[test]
    fn the_rail_holds_settings_with_its_four_sub_sections() {
        // covers: AC-1, AC-2.
        let view = view();
        assert_eq!(
            destinations(&view.items),
            vec![
                SETTINGS,
                SETTINGS_DICTATION,
                SETTINGS_LANGUAGES,
                SETTINGS_VOCABULARY,
                SETTINGS_TRANSCRIPTION
            ]
        );
    }

    #[test]
    fn the_rail_holds_no_section_that_does_not_exist() {
        // covers: AC-2. A section is not in the rail at all until its own
        // plan row lands and amends record 0004. If one appears here without a
        // screen behind it, this fails rather than shipping a nav item that
        // opens nothing. Languages and Vocabulary were on this list until
        // 2026-09-04 and came off it when records 0006 and 0005 built their
        // screens; History is plan row 5's and is still to come.
        // History is the one left. When plan row 5 lands it comes off this
        // check, exactly as Languages and Vocabulary did on 2026-09-04, and
        // anything else unbuilt joins it here.
        let view = view();
        {
            let unbuilt = "history";
            assert!(
                !destinations(&view.items)
                    .iter()
                    .any(|id| id.contains(unbuilt)),
                "the rail offers {unbuilt}, which has no screen behind it \
                 (record 0004 AC-2)"
            );
        }
    }

    #[test]
    fn the_landing_destination_is_one_of_the_rails_own_items() {
        // covers: AC-1. A landing the rail does not hold would leave the
        // dashboard opening on nothing.
        let view = view();
        assert!(destinations(&view.items).contains(&view.landing));
        assert_eq!(view.landing, SETTINGS_DICTATION);
    }

    #[test]
    fn the_rail_hands_out_no_wording() {
        // covers: AC-2. Identifiers only. What a person reads comes from
        // design/registry.md, so a sentence appearing in this payload would be
        // Rust inventing wording it does not own.
        let json = serde_json::to_string(&view()).unwrap();
        for word in [
            "Settings",
            "Dictation",
            "Languages",
            "Vocabulary",
            "Transcription",
        ] {
            assert!(
                !json.contains(word),
                "the rail payload carries the wording {word}. Rust hands out \
                 identifiers and the screen decides what a person reads"
            );
        }
    }
}
