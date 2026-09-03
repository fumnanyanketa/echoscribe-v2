//! The dictate feature's settings surface: the hotkey choice and the sound
//! switch (record 0002 AC-19, AC-21, AC-22).
//!
//! Four commands, and they are the whole of what the interface may do with
//! either setting. As everywhere else on this surface, the interface asks and
//! Rust decides: no command takes an account id, because Rust already knows it
//! from the session, and every one of them refuses when nobody is signed in.
//!
//! **The two hotkeys are this file's to hand out and nobody's to widen.**
//! `get_hotkey` returns both allowed values along with the chosen one, so the
//! interface renders a list rather than deciding for itself what is allowed,
//! and `set_hotkey` refuses anything that is not one of the two outright. That
//! is AC-22, and it is not a convenience: record 0002's Risk section allows
//! only two modifier double taps because any wider set would mean the keyboard
//! hook inspecting ordinary keystrokes all day. A third choice arriving here
//! would be a security change wearing a settings hat.
//!
//! **Both settings take effect immediately.** The sound switch does so on its
//! own, because `sound.rs` is handed the stored value at each open and close.
//! The hotkey needs one more step: the hook is armed with the chosen modifier,
//! so changing it re-arms the hook there and then, which is AC-19's "works
//! immediately, with no restart".
//!
//! The screen these four answer is not built yet. `design/registry.md` has no
//! control for choosing between two values and no switch, so building one
//! would be inventing design during a build. `/canvas` owns that, and this
//! file is deliberately finished ahead of it, the way milestone 3 stored the
//! last four characters of the key before anything displayed them.

use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::store::Hotkey;
use super::{hook, modifier_of, Dictate};

/// What the interface needs to draw the hotkey setting: which one is in force,
/// and the complete list it may be chosen from (record 0002 AC-19, AC-22).
///
/// The values are the stored ones, not wording. What a person reads for each is
/// the screen's business and is not decided here.
#[derive(Debug, Clone, Serialize)]
pub struct HotkeyChoice {
    chosen: &'static str,
    choices: Vec<&'static str>,
}

/// The chosen hotkey and the two it may be chosen from.
#[tauri::command]
pub fn get_hotkey(app: AppHandle) -> Result<HotkeyChoice, &'static str> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err("not_signed_in");
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err("not_ready");
    };
    let setting = state.store.setting_for(&account_id).map_err(|e| {
        eprintln!("dictate: could not read the chosen hotkey: {e}");
        "could_not_read"
    })?;

    Ok(HotkeyChoice {
        chosen: setting.hotkey.as_stored(),
        choices: Hotkey::CHOICES.iter().map(|h| h.as_stored()).collect(),
    })
}

/// Choose one of the two hotkeys (record 0002 AC-19, AC-22).
///
/// Refusing a value that is neither is the only refusal this command has left
/// to make, and it makes it before anything is written or armed. Treat the
/// incoming string as hostile input: it is text from outside the core, and the
/// only thing done with it is matching it against the two known values.
///
/// The hook is re-armed after the write, not before, so a database that would
/// not take the change cannot leave the hook watching a key the settings screen
/// will not show as chosen.
#[tauri::command]
pub fn set_hotkey(app: AppHandle, binding: String) -> Result<(), &'static str> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err("not_signed_in");
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err("not_ready");
    };
    let Some(hotkey) = Hotkey::from_chosen(&binding) else {
        // Nothing from `binding` is printed. It came from outside and it has no
        // business in a log; which of the two it was not is not information.
        eprintln!("dictate: a hotkey that is not one of the two was refused");
        return Err("unknown_hotkey");
    };

    let now = crate::sign_in::clock::now_iso8601();
    state
        .store
        .save_hotkey(&account_id, hotkey, &now)
        .map_err(|e| {
            eprintln!("dictate: could not save the chosen hotkey: {e}");
            "could_not_save"
        })?;

    // AC-19: in force from now, with no restart. The hook is already installed
    // and armed for this account, and this points it at the other modifier.
    hook::arm(modifier_of(hotkey));
    Ok(())
}

/// Whether the opening and closing sounds play (record 0002 AC-21). On for a
/// new account.
#[tauri::command]
pub fn get_dictation_sounds(app: AppHandle) -> Result<bool, &'static str> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err("not_signed_in");
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err("not_ready");
    };
    state
        .store
        .setting_for(&account_id)
        .map(|setting| setting.sounds_enabled)
        .map_err(|e| {
            eprintln!("dictate: could not read the sound setting: {e}");
            "could_not_read"
        })
}

/// The one sound switch (record 0002 AC-21).
///
/// Switched off, both sounds are silent and nothing else about dictation
/// changes: the pill still appears, because it is the one visible sign that the
/// microphone is open and AGENTS.md forbids opening it without one. Nothing
/// here touches the pill, and nothing may be added that does.
#[tauri::command]
pub fn set_dictation_sounds(app: AppHandle, enabled: bool) -> Result<(), &'static str> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err("not_signed_in");
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err("not_ready");
    };

    let now = crate::sign_in::clock::now_iso8601();
    state
        .store
        .save_sounds_enabled(&account_id, enabled, &now)
        .map_err(|e| {
            eprintln!("dictate: could not save the sound setting: {e}");
            "could_not_save"
        })
}

#[cfg(test)]
mod tests {
    /// This file's own source, minus its tests, flattened so a guard survives a
    /// formatter moving a call across lines.
    fn this_file_flattened() -> String {
        include_str!("settings.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    #[test]
    fn changing_the_hotkey_re_arms_the_hook() {
        // covers: AC-19, as a source guard only. "Picking the other one works
        // immediately, with no restart" is not the database write: the hook is
        // armed with one modifier and keeps watching it until something says
        // otherwise. Without this line the new choice is stored, the settings
        // screen shows it, and the old key is still the one that dictates until
        // the app restarts. Proving it for real needs a running app and a real
        // double tap, which is `/check verify`'s; this guard only stops the
        // line being tidied away as a duplicate of the write above it.
        assert!(
            this_file_flattened().contains("hook::arm(modifier_of(hotkey))"),
            "set_hotkey no longer re-arms the keyboard hook. The new hotkey \
             would then be stored and shown but not in force until a restart \
             (record 0002 AC-19)"
        );
    }

    #[test]
    fn no_command_here_takes_an_account_id() {
        // covers: AGENTS.md data rules, and record 0002's interface surface.
        // No command on this surface takes an account id, because Rust knows
        // it from the session. One that accepted one would let the interface
        // name whose settings to change.
        let flat = this_file_flattened();
        assert!(
            !flat.contains("account_id:String") && !flat.contains("accountid:String"),
            "a command in settings.rs now takes an account id. Whose settings \
             these are is the session's answer, never the interface's"
        );
        // Every command reads it from the session instead, and there are four.
        assert_eq!(
            flat.matches("crate::sign_in::account_id_from(&app)")
                .count(),
            4,
            "settings.rs no longer reads the account from the session in all \
             four of its commands. Each one must refuse when nobody is signed \
             in (record 0002, interface surface)"
        );
    }
}
