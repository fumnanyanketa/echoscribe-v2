mod dictate;
mod history;
mod language;
mod shell;
mod sign_in;
mod vocabulary;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Without this the dictation hotkey goes dead whenever an EchoScribe
        // window is focused. Tauri's windowing layer registers for Windows raw
        // input while one of its windows has focus, and that registration stops
        // this process's low-level keyboard hook being called at all, proven
        // live on 2026-08-30 (tauri-apps/tauri#13919). Nothing here uses device
        // events, so they are filtered always.
        .device_event_filter(tauri::DeviceEventFilter::Always)
        .setup(|app| {
            sign_in::init(app)?;
            // Both of these open their own connection to the same file and
            // both reference `account`, so they follow sign-in. Both are read
            // by the dictate feature at the moment a dictation starts, through
            // one named function each, so they are up before it can be asked
            // (records 0005 and 0006).
            language::init(app)?;
            vocabulary::init(app)?;
            dictate::init(app)?;
            // After the dictate feature, and the order matters: this one
            // reads `dictation` and creates nothing, and the dictate feature
            // is what owns and creates that table (record 0007).
            history::init(app)?;
            // Last of the three, and the order matters: the shell's table
            // references `account`, which sign-in creates, and it counts rows
            // in `deepgram_credential`, which dictate creates. It is also what
            // puts the first window on screen, because neither window is
            // visible until it decides which one belongs there
            // (docs/decisions/0004-the-app-shell.md).
            shell::init(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sign_in::get_auth_state,
            sign_in::start_sign_in,
            sign_in::cancel_sign_in,
            sign_in::sign_out,
            dictate::get_dictation_state,
            dictate::open_microphone_privacy_settings,
            dictate::retry_dictation,
            dictate::deepgram_key::save_deepgram_key,
            dictate::deepgram_key::get_deepgram_key_info,
            dictate::deepgram_key::clear_deepgram_key,
            dictate::deepgram_key::open_deepgram_signup,
            dictate::deepgram_key::open_deepgram_console,
            dictate::settings::get_hotkey,
            dictate::settings::set_hotkey,
            dictate::settings::get_dictation_sounds,
            dictate::settings::set_dictation_sounds,
            language::get_transcription_language,
            language::set_transcription_language,
            vocabulary::get_vocabulary,
            vocabulary::add_vocabulary_term,
            vocabulary::remove_vocabulary_term,
            shell::get_rail,
            history::get_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    /// This file's own source, minus its tests, flattened so the guard
    /// survives a formatter moving the builder call across lines.
    fn this_file_flattened() -> String {
        include_str!("lib.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    #[test]
    fn device_events_are_filtered_so_the_keyboard_hook_stays_alive() {
        // covers: AC-1, AC-5, as a source guard only. Without this builder
        // line the dictation hotkey goes dead exactly while an EchoScribe
        // window is focused: Tauri's windowing layer registers for Windows
        // raw input when one of its windows has focus, and that registration
        // stops this process's low-level keyboard hook being called at all,
        // proven live on 2026-08-30 (tauri-apps/tauri#13919). The behaviour
        // itself needs a running app, a focused EchoScribe window and a real
        // key press, so /check verify owns proving it live; this guard only
        // stops the line being tidied away as boilerplate, which is exactly
        // what it looks like.
        assert!(
            this_file_flattened()
                .contains(".device_event_filter(tauri::DeviceEventFilter::Always)"),
            "lib.rs no longer filters device events to Always. The dictation \
             hotkey will go dead whenever an EchoScribe window has focus \
             (2026-08-30, tauri-apps/tauri#13919)"
        );
    }

    /// Every feature that opens the database, in the order `run` boots them,
    /// paired with its own source.
    ///
    /// Each of the six declares its own `const DB_FILE`, because feature
    /// folders do not import from each other (AGENTS.md). That is the rule
    /// working as intended for behaviour, and it is six chances to be wrong
    /// about one fact, which is what standing rule 19 is about.
    const FEATURES: [(&str, &str); 6] = [
        ("sign_in", include_str!("sign_in/mod.rs")),
        ("language", include_str!("language/mod.rs")),
        ("vocabulary", include_str!("vocabulary/mod.rs")),
        ("dictate", include_str!("dictate/mod.rs")),
        ("history", include_str!("history/mod.rs")),
        ("shell", include_str!("shell/mod.rs")),
    ];

    /// The file name a feature joins to `app_data_dir()`, read out of that
    /// feature's own source.
    ///
    /// Read as text rather than named as a constant because every one of the
    /// six is private to its own module, so no test can see them all. Reading
    /// the source is what is available, and it is what the guard needs: the
    /// question is whether the six lines agree with each other.
    fn db_file_of<'a>(feature: &str, source: &'a str) -> &'a str {
        const DECLARATION: &str = "const DB_FILE: &str = \"";
        let opens = source.find(DECLARATION).unwrap_or_else(|| {
            panic!(
                "{feature}/mod.rs no longer declares `const DB_FILE: &str = \"...\"`. \
                 If the constant moved somewhere shared, this guard is the thing \
                 that has to follow it, because it is all that holds the six \
                 features to one database file"
            )
        }) + DECLARATION.len();
        let rest = &source[opens..];
        let closes = rest
            .find('"')
            .unwrap_or_else(|| panic!("{feature}/mod.rs has an unterminated DB_FILE literal"));
        &rest[..closes]
    }

    #[test]
    fn all_six_features_open_exactly_one_database_file() {
        // covers: AGENTS.md standing rule 19, and the blocker in
        // docs/reviews/2026-09-11-master.md. Records 0005, 0006 and 0007 are
        // the three that stop working when this fails.
        //
        // The history of it: sign_in, dictate and shell said
        // `echoscribe.sqlite3` and history, language and vocabulary said
        // `echoscribe.db`, so the app opened two files in the same directory
        // while every comment in it promised one. The `account` and
        // `dictation` tables are created only in the first, so history could
        // not be read at all, and neither a custom word nor a language choice
        // could be saved. History is the screen the app lands on after
        // sign-in, which made that the first thing a signed-in person saw on
        // every launch. Found by /check review on a second model on
        // 2026-09-11, after surviving 342 passing tests and a clean clippy.
        //
        // Why no other test saw it, and why this one is written the way it
        // is: every store's own tests open an in-memory database or their own
        // temp file and hand-create the tables they expect inside the
        // connection under test. So each feature is proved against a file
        // shaped the way that feature imagines, and never against the file it
        // actually gets. Six more unit tests of that shape would have found
        // nothing. The fact that needed a test was the one nothing owned:
        // that the six agree.
        //
        // This is a source guard and not a boot of the six `init` functions,
        // which is what it should have been. Booting them is not available to
        // `cargo test` today, for reasons written up beside this test in the
        // hand-off: `app_data_dir()` takes no injection point, so the six
        // cannot be aimed at a temporary directory without changing
        // application code, and three of the six `init`s install a Windows
        // keyboard hook, create a WebView window and start a thread that
        // talks to Clerk. So this guard proves the six lines agree. It does
        // not prove the one file they agree on ends up holding every table,
        // which still wants the real boot.
        let names: Vec<(&str, &str)> = FEATURES
            .iter()
            .map(|(feature, source)| (*feature, db_file_of(feature, source)))
            .collect();

        let mut distinct: Vec<&str> = names.iter().map(|(_, file)| *file).collect();
        distinct.sort_unstable();
        distinct.dedup();

        let inventory = names
            .iter()
            .map(|(feature, file)| format!("{feature} opens {file}"))
            .collect::<Vec<_>>()
            .join(", ");

        assert_eq!(
            distinct.len(),
            1,
            "EchoScribe opens {} different database files and must open exactly \
             one ({inventory}). All persistent data lives in one SQLite file on \
             the person's own machine (AGENTS.md data rules), and the tables one \
             feature creates are the tables another reads: the `account` table is \
             created by sign_in and the `dictation` table by dictate, so any \
             feature pointed at a different file finds neither. The last time \
             these drifted, history could not be read and neither a custom word \
             nor a language choice could be saved",
            distinct.len()
        );
    }
}
