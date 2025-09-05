use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: "https://api.groq.com/openai/v1".to_string(),
            model: "whisper-large-v3-turbo".to_string(),
        }
    }
}

pub struct SettingsManager<R: tauri::Runtime> {
    store: Arc<tauri_plugin_store::Store<R>>,
}

impl<R: tauri::Runtime> SettingsManager<R> {
    pub fn new(app: &tauri::AppHandle<R>) -> Result<Self, Box<dyn std::error::Error>> {
        let store = app.store("settings.json")?;
        Ok(Self { store })
    }

    pub fn load(&self) -> AppSettings {
        self.store
            .get("settings")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        self.store.set("settings".to_string(), serde_json::to_value(settings)?);
        self.store.save()?;
        Ok(())
    }
}