pub mod audio;
mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(audio::engine::AudioEngine::new(app.handle().clone())?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::process_wav,
            commands::list_audio_devices,
            commands::start_audio,
            commands::stop_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vocal Chain Discord");
}
