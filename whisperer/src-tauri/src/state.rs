use crate::{
    audio::AudioManager,
    settings::SettingsManager,
    transcription::TranscriptionService,
};
use std::sync::Arc;
use tokio::sync::Mutex;

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
}