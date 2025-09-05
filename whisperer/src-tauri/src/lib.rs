use tauri::Manager;

// Re-export all modules needed for both desktop and mobile
mod audio;
mod commands;
mod settings;
mod state;
mod storage;
mod transcription;

#[cfg(target_os = "android")]
mod android_audio;
#[cfg(target_os = "linux")]
mod linux_audio;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize settings manager
            let settings_manager = crate::settings::SettingsManager::new(app.handle())?;
            let settings = settings_manager.load();

            // Initialize audio recorder based on platform
            #[cfg(target_os = "linux")]
            let audio_recorder = {
                use crate::linux_audio::linux::LinuxAudioRecorder;
                Box::new(LinuxAudioRecorder::new()?) as Box<dyn crate::audio::AudioRecorder>
            };

            #[cfg(target_os = "android")]
            let audio_recorder = {
                use crate::android_audio::android::AndroidAudioRecorder;
                let api = app.state::<tauri::plugin::PluginApi>();
                Box::new(AndroidAudioRecorder::new(app.handle(), api.inner().clone())?) 
                    as Box<dyn crate::audio::AudioRecorder>
            };

            #[cfg(not(any(target_os = "linux", target_os = "android")))]
            compile_error!("Unsupported platform");

            let audio_manager = crate::audio::AudioManager::new(audio_recorder);

            // Create app state
            let app_state = crate::state::AppState::new(audio_manager, settings_manager);

            // TODO: Initialize transcription service on startup if API key exists
            // ISSUE: AppState can't derive Clone because SettingsManager<R> contains 
            // tauri_plugin_store::Store<R> which doesn't implement Clone.
            //
            // WHAT WAS TRIED:
            // 1. Deriving Clone on AppState to spawn async task for initialization
            // 2. Moving the initialization into the setup closure
            //
            // CURRENT WORKAROUND:
            // The transcription service is lazily initialized on first use in the
            // transcribe_recording command. This means the first transcription might
            // be slightly slower as it needs to create the service.
            //
            // PROPER SOLUTION:
            // Either refactor to initialize synchronously in setup, or create a
            // separate initialization command that the frontend can call on startup.

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::start_recording,
            crate::commands::stop_recording,
            crate::commands::pause_recording,
            crate::commands::resume_recording,
            crate::commands::get_recording_state,
            crate::commands::transcribe_recording,
            crate::commands::list_recordings,
            crate::commands::delete_recording,
            crate::commands::get_settings,
            crate::commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}