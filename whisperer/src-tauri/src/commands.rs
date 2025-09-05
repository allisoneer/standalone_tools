use crate::{
    settings::AppSettings,
    storage::{Recording, StorageManager, Transcription},
    state::AppState,
};
use chrono::Utc;
use tauri::State;

#[tauri::command]
pub async fn start_recording(
    state: State<'_, AppState<tauri::Wry>>,
) -> Result<(), String> {
    let audio_manager = state.audio_manager.lock().await;
    let mut recorder = audio_manager.recorder.lock().await;
    recorder.start_recording()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_recording(
    app: tauri::AppHandle<tauri::Wry>,
    state: State<'_, AppState<tauri::Wry>>,
) -> Result<Recording, String> {
    let audio_manager = state.audio_manager.lock().await;
    let mut recorder = audio_manager.recorder.lock().await;
    let audio_data = recorder.stop_recording()
        .await
        .map_err(|e| e.to_string())?;

    // Save audio file
    let filename = StorageManager::save_audio(&app, &audio_data, "wav")
        .map_err(|e| e.to_string())?;

    // Create recording entry
    let recording = Recording {
        id: uuid::Uuid::new_v4().to_string(),
        filename,
        duration_seconds: None, // TODO: Calculate from audio data
        // To implement: Parse WAV header to get sample count, divide by sample rate
        // For 16-bit mono at 16kHz: duration = (audio_data.len() - 44) / 2 / 16000
        // where 44 is the WAV header size and we divide by 2 for 16-bit samples
        created_at: Utc::now(),
        transcription: None,
    };

    // Update metadata
    let mut recordings = StorageManager::list_recordings(&app)
        .map_err(|e| e.to_string())?;
    recordings.push(recording.clone());
    StorageManager::save_metadata(&app, &recordings)
        .map_err(|e| e.to_string())?;

    Ok(recording)
}

#[tauri::command]
pub async fn pause_recording(
    state: State<'_, AppState<tauri::Wry>>,
) -> Result<(), String> {
    let audio_manager = state.audio_manager.lock().await;
    let mut recorder = audio_manager.recorder.lock().await;
    recorder.pause_recording()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_recording(
    state: State<'_, AppState<tauri::Wry>>,
) -> Result<(), String> {
    let audio_manager = state.audio_manager.lock().await;
    let mut recorder = audio_manager.recorder.lock().await;
    recorder.resume_recording()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recording_state(
    state: State<'_, AppState<tauri::Wry>>,
) -> Result<String, String> {
    let audio_manager = state.audio_manager.lock().await;
    let recorder = audio_manager.recorder.lock().await;
    let recording_state = recorder.get_state();
    Ok(format!("{:?}", recording_state))
}

#[tauri::command]
pub async fn transcribe_recording(
    app: tauri::AppHandle<tauri::Wry>,
    state: State<'_, AppState<tauri::Wry>>,
    recording_id: String,
) -> Result<Recording, String> {
    // Get recording metadata
    let mut recordings = StorageManager::list_recordings(&app)
        .map_err(|e| e.to_string())?;
    
    let recording_index = recordings.iter()
        .position(|r| r.id == recording_id)
        .ok_or("Recording not found")?;

    let mut recording = recordings[recording_index].clone();

    // Check if already transcribed
    if recording.transcription.is_some() {
        return Ok(recording);
    }

    // Get transcription service
    let service_lock = state.transcription_service.lock().await;
    let service = service_lock.as_ref()
        .ok_or("Transcription service not configured. Please set API key in settings.")?;

    // Load audio file
    let recordings_dir = StorageManager::recordings_dir(&app)
        .map_err(|e| e.to_string())?;
    let audio_path = recordings_dir.join(&recording.filename);
    let audio_data = std::fs::read(&audio_path)
        .map_err(|e| e.to_string())?;

    // Get current model from settings
    let settings_manager = state.settings_manager.lock().await;
    let settings = settings_manager.load();
    
    // Transcribe
    let (text, metadata) = service
        .transcribe_with_metadata(
            audio_data,
            recording.filename.clone(),
            settings.model.clone(),
            true, // Include timestamps
        )
        .await
        .map_err(|e| e.to_string())?;

    // Update recording with transcription
    recording.transcription = Some(Transcription {
        text,
        language: metadata.get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        model_used: settings.model,
        created_at: Utc::now(),
        metadata: Some(metadata),
    });

    // Save updated metadata
    recordings[recording_index] = recording.clone();
    StorageManager::save_metadata(&app, &recordings)
        .map_err(|e| e.to_string())?;

    Ok(recording)
}

#[tauri::command]
pub async fn list_recordings(
    app: tauri::AppHandle<tauri::Wry>,
) -> Result<Vec<Recording>, String> {
    StorageManager::list_recordings(&app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_recording(
    app: tauri::AppHandle<tauri::Wry>,
    recording_id: String,
) -> Result<(), String> {
    let mut recordings = StorageManager::list_recordings(&app)
        .map_err(|e| e.to_string())?;

    let recording_index = recordings.iter()
        .position(|r| r.id == recording_id)
        .ok_or("Recording not found")?;

    let recording = &recordings[recording_index];
    
    // Delete audio file
    let recordings_dir = StorageManager::recordings_dir(&app)
        .map_err(|e| e.to_string())?;
    let audio_path = recordings_dir.join(&recording.filename);
    if audio_path.exists() {
        std::fs::remove_file(audio_path)
            .map_err(|e| e.to_string())?;
    }

    // Remove from metadata
    recordings.remove(recording_index);
    StorageManager::save_metadata(&app, &recordings)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState<tauri::Wry>>,
) -> Result<AppSettings, String> {
    let settings_manager = state.settings_manager.lock().await;
    Ok(settings_manager.load())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState<tauri::Wry>>,
    settings: AppSettings,
) -> Result<(), String> {
    // Validate settings
    if let Some(api_key) = &settings.api_key {
        if !api_key.is_empty() {
            // Update transcription service
            state.update_transcription_service(
                api_key.clone(),
                settings.base_url.clone(),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    // Save settings
    let settings_manager = state.settings_manager.lock().await;
    settings_manager.save(&settings)
        .map_err(|e| e.to_string())
}