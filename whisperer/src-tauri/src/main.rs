// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// TODO: This file has significant duplication with lib.rs
// Both files contain the same module declarations and initialization logic.
// This is because:
// - main.rs is used for desktop builds
// - lib.rs is used for mobile builds (with #[cfg_attr(mobile, tauri::mobile_entry_point)])
//
// Ideally, we would refactor the common initialization logic into a shared function
// to avoid maintaining two copies of the same code.

mod settings;
mod storage;
mod audio;
mod transcription;
mod state;
mod commands;

#[cfg(target_os = "android")]
mod android_audio;
#[cfg(target_os = "linux")]
mod linux_audio;

use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize settings manager
            let settings_manager = settings::SettingsManager::new(app.handle())?;
            let settings = settings_manager.load();

            // Initialize audio recorder based on platform
            #[cfg(target_os = "linux")]
            let audio_recorder = {
                use crate::linux_audio::linux::LinuxAudioRecorder;
                Box::new(LinuxAudioRecorder::new()?) as Box<dyn audio::AudioRecorder>
            };

            #[cfg(target_os = "android")]
            let audio_recorder = {
                use crate::android_audio::android::AndroidAudioRecorder;
                let api = app.state::<tauri::plugin::PluginApi<_>>();
                Box::new(AndroidAudioRecorder::new(app.handle(), api.inner().clone())?) 
                    as Box<dyn audio::AudioRecorder>
            };

            #[cfg(not(any(target_os = "linux", target_os = "android")))]
            compile_error!("Unsupported platform");

            let audio_manager = audio::AudioManager::new(audio_recorder);

            // Create app state
            let app_state = AppState::new(audio_manager, settings_manager);

            // Initialize transcription service if API key exists
            if let Some(api_key) = settings.api_key {
                if !api_key.is_empty() {
                    use crate::transcription::TranscriptionService;
                    let service = TranscriptionService::new(api_key, settings.base_url);
                    *app_state.transcription_service.blocking_lock() = Some(service);
                }
            }

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::pause_recording,
            commands::resume_recording,
            commands::get_recording_state,
            commands::transcribe_recording,
            commands::list_recordings,
            commands::delete_recording,
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
