use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingSource {
    Recorded,
    Uploaded,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recording {
    pub id: String,
    pub filename: String,
    pub duration_seconds: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub transcription: Option<Transcription>,
    #[serde(default = "default_source")]
    pub source: RecordingSource,
    pub original_filename: Option<String>,
    pub original_format: Option<String>,
}

fn default_source() -> RecordingSource {
    RecordingSource::Recorded
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transcription {
    pub text: String,
    pub language: Option<String>,
    pub model_used: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

pub struct StorageManager;

impl StorageManager {
    pub fn recordings_dir<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let app_dir = app.path().app_local_data_dir()?;
        let recordings_dir = app_dir.join("recordings");
        std::fs::create_dir_all(&recordings_dir)?;
        Ok(recordings_dir)
    }
    
    pub fn calculate_wav_duration(audio_data: &[u8]) -> Option<f64> {
        if audio_data.len() < 44 {
            return None;
        }
        
        // Parse WAV header to get sample rate and data size
        let sample_rate = u32::from_le_bytes([
            audio_data[24], audio_data[25], 
            audio_data[26], audio_data[27]
        ]);
        
        let data_size = u32::from_le_bytes([
            audio_data[40], audio_data[41], 
            audio_data[42], audio_data[43]
        ]);
        
        // Calculate duration: data_size / (sample_rate * bytes_per_sample * channels)
        // For 16-bit mono: bytes_per_sample = 2, channels = 1
        Some(data_size as f64 / (sample_rate as f64 * 2.0))
    }

    pub fn save_audio<R: tauri::Runtime>(
        app: &AppHandle<R>,
        audio_data: &[u8],
        format: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = uuid::Uuid::new_v4().to_string();
        let filename = format!("{}.{}", id, format);
        let recordings_dir = Self::recordings_dir(app)?;
        let file_path = recordings_dir.join(&filename);
        
        std::fs::write(file_path, audio_data)?;
        Ok(filename)
    }

    pub fn list_recordings<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<Vec<Recording>, Box<dyn std::error::Error>> {
        let recordings_dir = Self::recordings_dir(app)?;
        let metadata_path = recordings_dir.join("metadata.json");
        
        if metadata_path.exists() {
            let data = std::fs::read_to_string(metadata_path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn save_metadata<R: tauri::Runtime>(
        app: &AppHandle<R>,
        recordings: &[Recording],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let recordings_dir = Self::recordings_dir(app)?;
        let metadata_path = recordings_dir.join("metadata.json");
        let data = serde_json::to_string_pretty(recordings)?;
        std::fs::write(metadata_path, data)?;
        Ok(())
    }
}