#[cfg(target_os = "android")]
pub mod android {
    use crate::audio::{AudioRecorder, RecordingState};
    use async_trait::async_trait;
    use tauri::{AppHandle, Manager, Runtime};
    use tauri::plugin::PluginHandle;

    pub struct AndroidAudioRecorder<R: Runtime> {
        plugin_handle: PluginHandle<R>,
        state: RecordingState,
    }

    impl<R: Runtime> AndroidAudioRecorder<R> {
        pub fn new(app: &AppHandle<R>) -> Result<Self, Box<dyn std::error::Error>> {
            // Get the audio plugin handle from app state
            let audio_handle = app
                .try_state::<tauri_plugin_audio::mobile::AudioPluginHandle<R>>()
                .ok_or("Audio plugin not initialized")?;
            
            Ok(Self {
                plugin_handle: audio_handle.0.clone(), // Access inner PluginHandle
                state: RecordingState::Idle,
            })
        }
    }

    #[async_trait]
    impl<R: Runtime> AudioRecorder for AndroidAudioRecorder<R> {
        async fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.plugin_handle
                .run_mobile_plugin_async::<()>("startRecording", ())
                .await?;
            self.state = RecordingState::Recording;
            Ok(())
        }

        async fn stop_recording(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let data = self.plugin_handle
                .run_mobile_plugin_async::<Vec<u8>>("stopRecording", ())
                .await?;
            self.state = RecordingState::Idle;
            Ok(data)
        }

        async fn pause_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.plugin_handle
                .run_mobile_plugin_async::<()>("pauseRecording", ())
                .await?;
            self.state = RecordingState::Paused;
            Ok(())
        }

        async fn resume_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.plugin_handle
                .run_mobile_plugin_async::<()>("resumeRecording", ())
                .await?;
            self.state = RecordingState::Recording;
            Ok(())
        }

        fn get_state(&self) -> RecordingState {
            self.state
        }
    }
}
