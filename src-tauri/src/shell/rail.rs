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
//! The list holds only sections that work, and **from 2026-09-04 that is every
//! section this app has**: History first, then Settings with Dictation,
//! Languages, Vocabulary and Transcription beneath it, in the comp's order.
//! Each of plan rows 3, 4 and 5 added its own item by an amendment to record
//! 0004 as it landed. Languages and Vocabulary joined by the third amendment of
//! 2026-09-04, for records 0006 and 0005, and **History joined by the fourth,
//! for record 0007, which also moved the landing destination to it.**
//!
//! The rule that let each of them join has nothing left to exclude today and is
//! not retired: nothing unbuilt is ever in this list, not even disabled, because
//! `design/registry.md` draws `Nav item` with exactly two states, so a disabled
//! third does not exist in this design system, and a nav item that opens nothing
//! is a promise the app cannot keep.

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
    /// The landing destination. **History, from 2026-09-04**, by record 0004's
    /// fourth amendment of that day for record 0007, which also reworded that
    /// record's AC-1. It was Settings, on Dictation, until then, and that
    /// record always said so was temporary: it was the only section that
    /// existed, so it was the only honest landing.
    pub landing: &'static str,
}

/// Every identifier the rail can ever hand out today. One place, so a reader
/// can see the whole list at once and so the tests below cannot drift from it.
pub const HISTORY: &str = "history";
pub const SETTINGS: &str = "settings";
pub const SETTINGS_DICTATION: &str = "settings.dictation";
pub const SETTINGS_LANGUAGES: &str = "settings.languages";
pub const SETTINGS_VOCABULARY: &str = "settings.vocabulary";
pub const SETTINGS_TRANSCRIPTION: &str = "settings.transcription";

/// The rail as it stands. A literal, so "the rail being unreadable" is not a
/// failure that can happen (record 0004, Interface surface).
pub fn view() -> RailView {
    RailView {
        items: vec![
            // History first, and a top level item rather than one of Settings'
            // children: it has no children of its own, it is not a setting,
            // and it is where the dashboard lands (record 0004, fourth
            // amendment of 2026-09-04).
            RailItem {
                id: HISTORY,
                children: Vec::new(),
            },
            RailItem {
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
            },
        ],
        landing: HISTORY,
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
    fn the_rail_holds_history_and_settings_with_its_four_sub_sections() {
        // covers: AC-1, AC-2. The whole list, in order, in one assertion. This
        // is now the only thing keeping an unbuilt section out of the rail: the
        // sentinel guard that used to sit below named "history" as the one
        // section with no screen, and it was deleted in the same change that
        // gave History a screen, per standing rule 11. Nothing unbuilt is left
        // for it to name, and an item appearing here without a screen fails
        // this test instead.
        let view = view();
        assert_eq!(
            destinations(&view.items),
            vec![
                HISTORY,
                SETTINGS,
                SETTINGS_DICTATION,
                SETTINGS_LANGUAGES,
                SETTINGS_VOCABULARY,
                SETTINGS_TRANSCRIPTION
            ]
        );
    }

    #[test]
    fn history_is_a_top_level_item_with_no_sub_sections() {
        // covers: AC-2, and record 0004's fourth amendment of 2026-09-04.
        // History is a sibling of Settings and not one of its children: it has
        // no children of its own and it is not a setting. Nested, dashboard.js
        // would resolve pressing it to a sub-section that does not exist and
        // the surface would draw nothing.
        let view = view();
        let history = view
            .items
            .iter()
            .find(|item| item.id == HISTORY)
            .expect("the rail holds History");
        assert!(history.children.is_empty());
    }

    #[test]
    fn the_landing_destination_has_a_screen_of_its_own() {
        // covers: AC-1, and record 0007's AC-1. The landing has to be a
        // destination that draws something itself rather than a section that
        // resolves to a child, because record 0004's AC-1 promises what a
        // person sees without clicking anything.
        let view = view();
        let landing = view
            .items
            .iter()
            .find(|item| item.id == view.landing)
            .expect("the landing is one of the rail's top level items");
        assert!(landing.children.is_empty());
    }

    #[test]
    fn the_landing_destination_is_one_of_the_rails_own_items() {
        // covers: AC-1. A landing the rail does not hold would leave the
        // dashboard opening on nothing.
        let view = view();
        assert!(destinations(&view.items).contains(&view.landing));
        assert_eq!(view.landing, HISTORY);
    }

    #[test]
    fn the_rail_hands_out_no_wording() {
        // covers: AC-2. Identifiers only. What a person reads comes from
        // design/registry.md, so a sentence appearing in this payload would be
        // Rust inventing wording it does not own.
        let json = serde_json::to_string(&view()).unwrap();
        for word in [
            "History",
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
