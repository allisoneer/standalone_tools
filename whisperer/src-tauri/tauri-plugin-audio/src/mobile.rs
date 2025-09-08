use tauri::{AppHandle, Runtime};
use tauri::plugin::{PluginApi, PluginHandle};
use serde::de::DeserializeOwned;

pub struct AudioPluginHandle<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> AudioPluginHandle<R> {
    pub async fn start_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.0.run_mobile_plugin::<()>("startRecording", ())?;
        Ok(())
    }
    
    pub async fn stop_recording(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(self.0.run_mobile_plugin("stopRecording", ())?)
    }
    
    pub async fn pause_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.0.run_mobile_plugin::<()>("pauseRecording", ())?;
        Ok(())
    }
    
    pub async fn resume_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.0.run_mobile_plugin::<()>("resumeRecording", ())?;
        Ok(())
    }
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<AudioPluginHandle<R>, Box<dyn std::error::Error>> {
    let handle = api.register_android_plugin("com.whisperer.audio", "AudioRecorderPlugin")?;
    Ok(AudioPluginHandle(handle))
}