mod dictate;
mod sign_in;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            dictate::get_dictation_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
