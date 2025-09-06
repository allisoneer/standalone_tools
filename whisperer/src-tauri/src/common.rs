use crate::{audio::AudioManager, settings::SettingsManager, state::{AppState, InitData}};
use tauri::{AppHandle, Runtime};

pub fn initialize_app_components<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(AppState<R>, InitData), Box<dyn std::error::Error>> {
    // Initialize settings manager
    let settings_manager = SettingsManager::new(app)?;
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
        Box::new(AndroidAudioRecorder::new(app)?) as Box<dyn crate::audio::AudioRecorder>
    };

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    compile_error!("Unsupported platform");

    let audio_manager = AudioManager::new(audio_recorder);

    // Create app state
    let app_state = AppState::new(audio_manager, settings_manager);

    // Create initialization data
    let init_data = InitData {
        api_key: settings.api_key.clone(),
        base_url: settings.base_url.clone(),
    };

    // Return both app_state and init_data
    Ok((app_state, init_data))
}

#[macro_export]
macro_rules! register_app_commands {
    () => {
        tauri::generate_handler![
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
        ]
    };
}