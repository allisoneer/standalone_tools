use crate::{
    audio::AudioManager,
    settings::SettingsManager,
    transcription::TranscriptionService,
};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;

pub struct InitData {
    pub api_key: Option<String>,
    pub base_url: String,
}

pub struct AppState<R: tauri::Runtime> {
    pub audio_manager: Arc<Mutex<AudioManager>>,
    pub settings_manager: Arc<Mutex<SettingsManager<R>>>,
    pub transcription_service: Arc<Mutex<Option<TranscriptionService>>>,
}

impl<R: tauri::Runtime> AppState<R> {
    pub fn new(
        audio_manager: AudioManager,
        settings_manager: SettingsManager<R>,
    ) -> Self {
        Self {
            audio_manager: Arc::new(Mutex::new(audio_manager)),
            settings_manager: Arc::new(Mutex::new(settings_manager)),
            transcription_service: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn update_transcription_service(
        &self,
        api_key: String,
        base_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service = TranscriptionService::new(api_key, base_url);
        *self.transcription_service.lock().await = Some(service);
        Ok(())
    }
    

    pub async fn initialize_transcription_async<T: Runtime>(
        app_handle: AppHandle<T>,
        api_key: String,
        base_url: Option<String>,
    ) -> Result<(), String> {
        // Create service in a panic-safe way
        let service = match std::panic::catch_unwind(|| {
            TranscriptionService::new(api_key, base_url.unwrap_or_default())
        }) {
            Ok(service) => service,
            Err(_) => {
                return Err("Failed to create transcription service".to_string());
            }
        };
        
        // Get app state and update service
        match app_handle.try_state::<AppState<T>>() {
            Some(state) => {
                *state.transcription_service.lock().await = Some(service);
                Ok(())
            }
            None => {
                Err("Failed to access app state".to_string())
            }
        }
    }
}