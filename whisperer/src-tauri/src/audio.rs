use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
}

#[async_trait]
pub trait AudioRecorder: Send + Sync {
    async fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn stop_recording(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    async fn pause_recording(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn resume_recording(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn get_state(&self) -> RecordingState;
}

pub struct AudioManager {
    pub recorder: Arc<Mutex<Box<dyn AudioRecorder>>>,
}

impl AudioManager {
    pub fn new(recorder: Box<dyn AudioRecorder>) -> Self {
        Self {
            recorder: Arc::new(Mutex::new(recorder)),
        }
    }
}