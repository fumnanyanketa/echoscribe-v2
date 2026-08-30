mod dictate;
mod sign_in;

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
            dictate::init(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sign_in::get_auth_state,
            sign_in::start_sign_in,
            sign_in::cancel_sign_in,
            sign_in::sign_out,
            dictate::get_dictation_state,
            dictate::open_microphone_privacy_settings,
            dictate::retry_dictation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
