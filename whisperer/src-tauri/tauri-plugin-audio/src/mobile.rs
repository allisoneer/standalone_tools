use tauri::{AppHandle, Runtime};
use tauri::plugin::{PluginApi, PluginHandle};

pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    api.register_android_plugin("com.whisperer.audio", "AudioRecorderPlugin")?;
    Ok(())
}