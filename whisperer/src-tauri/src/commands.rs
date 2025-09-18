use crate::{
    audio::AudioDevice,
    audio_processor::AudioProcessor,
    settings::AppSettings,
    storage::{Recording, RecordingSource, StorageManager, Transcription},
    state::AppState,
};
use chrono::Utc;
use tauri::State;
use cpal::traits::{DeviceTrait, HostTrait};

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

    // Calculate duration
    let duration = StorageManager::calculate_wav_duration(&audio_data);
    
    // Create recording entry
    let recording = Recording {
        id: uuid::Uuid::new_v4().to_string(),
        filename,
        duration_seconds: duration,
        created_at: Utc::now(),
        transcription: None,
        source: RecordingSource::Recorded,
        original_filename: None,
        original_format: None,
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
pub async fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let mut devices = vec![];
    
    // Get the default device name for comparison
    let default_device_name = host.default_input_device()
        .and_then(|d| d.name().ok());
    
    // Enumerate all input devices
    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Ok(name) = device.name() {
                // Only include devices that support input configurations
                if device.supported_input_configs().is_ok() {
                    devices.push(AudioDevice {
                        id: name.clone(),
                        name: name.clone(),
                        is_default: default_device_name.as_ref() == Some(&name),
                    });
                }
            }
        }
    }
    
    // Sort devices: default first, then alphabetically
    devices.sort_by(|a, b| {
        match (a.is_default, b.is_default) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    Ok(devices)
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
    let service = if let Some(svc) = service_lock.as_ref() {
        svc
    } else {
        // Check if API key exists in settings
        let settings_manager = state.settings_manager.lock().await;
        let settings = settings_manager.load();
        let error_msg = if settings.api_key.is_none() || settings.api_key.as_ref().unwrap().is_empty() {
            "Transcription service not configured. Please set API key in settings."
        } else {
            "Transcription service is initializing. Please try again in a moment."
        };
        return Err(error_msg.to_string());
    };

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
    if let Some(api_key) = &settings.api_key 
        && !api_key.is_empty() {
        // Update transcription service
        state.update_transcription_service(
            api_key.clone(),
            settings.base_url.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    // Save settings
    let settings_manager = state.settings_manager.lock().await;
    settings_manager.save(&settings)
        .map_err(|e| e.to_string())?;
    
    // Note: Audio device preference will be applied on next recording start
    // since we can't change device mid-recording
    
    Ok(())
}

#[tauri::command]
pub async fn upload_audio_file(
    app: tauri::AppHandle<tauri::Wry>,
    file_data: Vec<u8>,
    original_filename: String,
) -> Result<Recording, String> {
    // Detect format from filename
    let extension = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown");
    
    // Convert to WAV if needed
    let wav_data = if extension.eq_ignore_ascii_case("wav") {
        file_data
    } else {
        AudioProcessor::convert_to_wav(file_data, Some(&original_filename))
            .map_err(|e| format!("Failed to convert audio: {}", e))?
    };
    
    // Calculate duration
    let duration = StorageManager::calculate_wav_duration(&wav_data);
    
    // Save the converted file
    let filename = StorageManager::save_audio(&app, &wav_data, "wav")
        .map_err(|e| e.to_string())?;
    
    // Create recording entry
    let recording = Recording {
        id: uuid::Uuid::new_v4().to_string(),
        filename,
        duration_seconds: duration,
        created_at: Utc::now(),
        transcription: None,
        source: RecordingSource::Uploaded,
        original_filename: Some(original_filename.clone()),
        original_format: Some(extension.to_string()),
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
pub fn get_max_upload_size() -> u32 {
    25 * 1024 * 1024 // 25MB
}