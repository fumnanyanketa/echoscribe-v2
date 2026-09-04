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
}
