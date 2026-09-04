//! The languages EchoScribe offers, and the only ones it will ever ask
//! Deepgram for (record 0006 AC-1, AC-6).
//!
//! **A fixed list of literals, and that is a security control as much as a
//! feature.** The chosen language ends up in the query of the streaming
//! address, so a value like `en&redact=pci` reaching it would change what is
//! asked of Deepgram on every dictation, silently. Nothing but one of these 64
//! literals is ever sent: [`is_offered`] is what `set_transcription_language`
//! refuses on and what a stored value is read through. It is the same refusal
//! `set_hotkey` makes over two values, applied to a longer list.
//!
//! **The source is Deepgram's own table**, read on 2026-09-04 from
//! `https://developers.deepgram.com/docs/models-languages-overview`, and the
//! rule for reading it is record 0006's: one row per language Deepgram names,
//! using the first code its row gives, plus the five variants Deepgram's own
//! table names as distinct languages, plus `multi`. Regional variants of one
//! language are deliberately not offered; see that record.
//!
//! **No wording lives here.** Rust hands out codes and the screen decides what
//! a person reads, the same division `get_hotkey` and the rail are on. A test
//! at the bottom of `mod.rs` fails the build if this list can hand out a code
//! the screen has no name for.

/// English, and what a person gets until they choose otherwise. Exactly what
/// record 0002 had fixed in code, so this feature changes nothing for anybody
/// who does nothing.
pub const DEFAULT: &str = "en";

/// Deepgram's multilingual model, which moves between ten languages inside one
/// stream. Named here because it is the one entry that is not a language, and
/// because record 0006's AC-5 is about it specifically.
pub const MULTILINGUAL: &str = "multi";

/// Every language this app offers, in no particular order: the screen sorts by
/// the name a person reads, which is the only side that has those names.
///
/// 64 entries. Adding one means adding its wording to `src/language/language.js`
/// in the same change, and a test enforces that.
pub const OFFERED: [&str; 64] = [
    MULTILINGUAL,
    "af",    // Afrikaans
    "ar",    // Arabic
    "hy",    // Armenian
    "as",    // Assamese
    "be",    // Belarusian
    "bn",    // Bengali
    "bs",    // Bosnian
    "bg",    // Bulgarian
    "ca",    // Catalan
    "zh-HK", // Chinese (Cantonese, Traditional)
    "zh",    // Chinese (Mandarin, Simplified)
    "zh-TW", // Chinese (Mandarin, Traditional)
    "hr",    // Croatian
    "cs",    // Czech
    "da",    // Danish
    "nl",    // Dutch
    DEFAULT, // English
    "et",    // Estonian
    "fi",    // Finnish
    "nl-BE", // Flemish
    "fr",    // French
    "ka",    // Georgian
    "de",    // German
    "de-CH", // German (Switzerland)
    "el",    // Greek
    "gu",    // Gujarati
    "he",    // Hebrew
    "hi",    // Hindi
    "hu",    // Hungarian
    "id",    // Indonesian
    "it",    // Italian
    "ja",    // Japanese
    "kn",    // Kannada
    "kk",    // Kazakh
    "ko",    // Korean
    "lv",    // Latvian
    "lt",    // Lithuanian
    "mk",    // Macedonian
    "ms",    // Malay
    "mr",    // Marathi
    "mn",    // Mongolian
    "ne",    // Nepali
    "no",    // Norwegian
    "ps",    // Pashto
    "fa",    // Persian
    "pl",    // Polish
    "pt",    // Portuguese
    "pa",    // Punjabi
    "ro",    // Romanian
    "ru",    // Russian
    "sr",    // Serbian
    "sk",    // Slovak
    "sl",    // Slovenian
    "es",    // Spanish
    "sv",    // Swedish
    "tl",    // Tagalog
    "ta",    // Tamil
    "te",    // Telugu
    "th",    // Thai
    "tr",    // Turkish
    "uk",    // Ukrainian
    "ur",    // Urdu
    "vi",    // Vietnamese
];

/// Whether this app offers `code` at all. **For the guards only.**
///
/// The running app never asks this question on its own: it goes through
/// [`from_chosen`] or [`from_stored`], because both need the `&'static str`
/// back rather than a yes. This exists for the two tests that check the two
/// lists have not drifted apart, where a plain answer is what is wanted, and it
/// is compiled out of the app so that it cannot become a third gate somebody
/// forgets to keep in step with the other two.
#[cfg(test)]
pub fn is_offered(code: &str) -> bool {
    OFFERED.contains(&code)
}

/// A value the interface asked for, read strictly. `None` is
/// `set_transcription_language`'s one refusal, and it is deliberately not the
/// same reading as [`from_stored`]: a row this app cannot understand falls back
/// so dictation still works, but a write it cannot understand is refused
/// outright rather than quietly turned into English. Record 0002's `Hotkey`
/// draws exactly this distinction over two values.
pub fn from_chosen(code: &str) -> Option<&'static str> {
    OFFERED.iter().copied().find(|offered| *offered == code)
}

/// A stored value, read leniently. Anything this app does not offer is
/// [`DEFAULT`], never an instruction to the stream (record 0006's data rules,
/// and record 0002's rule for a stored hotkey it cannot understand).
pub fn from_stored(code: &str) -> &'static str {
    from_chosen(code).unwrap_or(DEFAULT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_the_size_the_record_says() {
        // covers: AC-1, AC-6. Record 0006 says 64, the screen holds 64 names,
        // and the count is the cheapest way to notice one going missing in a
        // merge.
        assert_eq!(OFFERED.len(), 64);
    }

    #[test]
    fn no_code_appears_twice() {
        // covers: AC-1. A duplicate would draw two identical rows, one of which
        // could never be chosen, because the chosen row is matched by code.
        let mut seen = OFFERED;
        seen.sort_unstable();
        let before = seen.len();
        let mut unique = seen.to_vec();
        unique.dedup();
        assert_eq!(before, unique.len(), "a language code is in the list twice");
    }

    #[test]
    fn english_and_multilingual_are_both_offered() {
        // covers: AC-1, AC-5. English is the default, so a list without it
        // would make every account fall back to a language it does not offer;
        // Multilingual is AC-5's whole subject.
        assert!(is_offered(DEFAULT));
        assert!(is_offered(MULTILINGUAL));
        assert_eq!(DEFAULT, "en");
        assert_eq!(MULTILINGUAL, "multi");
    }

    #[test]
    fn every_code_is_a_plain_language_tag_and_nothing_else() {
        // covers: AC-3, and record 0006's risk section. These strings go into
        // the query of the streaming address. A code holding an ampersand, an
        // equals sign or a space would be a second parameter smuggled into
        // every request, so the shape of every literal is checked here rather
        // than trusted to have been typed carefully.
        for code in OFFERED {
            assert!(!code.is_empty(), "an empty language code");
            assert!(
                code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "the language code {code:?} holds something other than letters, digits and a \
                 hyphen, which could change what is asked of Deepgram"
            );
            assert!(
                code.len() <= 8,
                "the language code {code:?} is not a language tag"
            );
        }
    }

    #[test]
    fn a_write_this_app_does_not_offer_is_refused_rather_than_defaulted() {
        // covers: AC-2, and record 0006's risk section. This is the refusal
        // that stops a string from the interface reaching Deepgram. Quietly
        // turning it into English would be worse than refusing: a person would
        // be told their choice was saved.
        for hostile in ["en&redact=pci", "EN", "en-GB", "", "multi ", "zz"] {
            assert_eq!(from_chosen(hostile), None, "{hostile:?} was accepted");
        }
        assert_eq!(from_chosen("en"), Some("en"));
        assert_eq!(from_chosen("zh-HK"), Some("zh-HK"));
    }

    #[test]
    fn a_stored_value_this_app_does_not_offer_reads_as_english() {
        // covers: AC-1, AC-3. The lenient half. A row holding something
        // unknown, from a hand edit or from a list that later shrank, must
        // leave dictation working rather than send Deepgram a value it will
        // refuse.
        for unknown in ["", "zz", "en-GB", "en&redact=pci"] {
            assert_eq!(from_stored(unknown), DEFAULT, "{unknown:?}");
        }
        assert_eq!(from_stored("ja"), "ja");
    }

    #[test]
    fn the_two_readings_agree_on_everything_this_app_offers() {
        // covers: AC-2, AC-3. The strict and the lenient reading differ only on
        // what is not offered. If they ever differed on something that is, a
        // person could store a language and have a different one sent.
        for code in OFFERED {
            assert_eq!(from_chosen(code), Some(code));
            assert_eq!(from_stored(code), code);
        }
    }

    #[test]
    fn no_wording_lives_in_this_file() {
        // covers: AC-1. `design/registry.md` owns what a person reads, and the
        // screen holds it, the same division as the rail's items and the two
        // hotkey rows. The comments here name each language so a reader can
        // follow the list, and a comment is not a payload: what matters is that
        // nothing outside a comment is a name.
        for code in OFFERED {
            assert!(
                !code.contains(' ') && code.len() <= 8,
                "the entry {code:?} looks like wording rather than a code. Rust hands out \
                 codes and the screen decides what a person reads"
            );
        }
    }
}
