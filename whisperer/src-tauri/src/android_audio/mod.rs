#[cfg(target_os = "android")]
pub mod android {
    use crate::audio::{AudioRecorder, RecordingState};
    use async_trait::async_trait;
    use tauri::{AppHandle, Runtime};

    pub struct AndroidAudioRecorder<R: Runtime> {
        app_handle: AppHandle<R>,
        state: RecordingState,
    }

    impl<R: Runtime> AndroidAudioRecorder<R> {
        pub fn new(app: &AppHandle<R>) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                app_handle: app.clone(),
                state: RecordingState::Idle,
            })
        }
    }

    #[async_trait]
    impl<R: Runtime> AudioRecorder for AndroidAudioRecorder<R> {
        async fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.app_handle
                .run_mobile_plugin("plugin:audio|startRecording", ())
                .await?;
            self.state = RecordingState::Recording;
            Ok(())
        }

        async fn stop_recording(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let data = self.app_handle
                .run_mobile_plugin("plugin:audio|stopRecording", ())
                .await?;
            self.state = RecordingState::Idle;
            Ok(data)
        }

        async fn pause_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.app_handle
                .run_mobile_plugin("plugin:audio|pauseRecording", ())
                .await?;
            self.state = RecordingState::Paused;
            Ok(())
        }

        async fn resume_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.app_handle
                .run_mobile_plugin("plugin:audio|resumeRecording", ())
                .await?;
            self.state = RecordingState::Recording;
            Ok(())
        }

        fn get_state(&self) -> RecordingState {
            self.state
        }
    }
}
