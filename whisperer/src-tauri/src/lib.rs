
// Re-export all modules needed for both desktop and mobile
mod audio;
mod commands;
mod common;
mod settings;
mod state;
mod storage;
mod transcription;

use tauri::Manager;

#[cfg(target_os = "android")]
mod android_audio;
#[cfg(target_os = "linux")]
mod linux_audio;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init());
    
    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(tauri_plugin_audio::init());
    }
    
    builder
        .setup(|app| {
            let (app_state, init_data) = common::initialize_app_components(app.handle())?;
            
            // Mobile can now also use eager initialization if desired
            app_state.initialize_transcription_blocking(init_data)?;
            
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(crate::register_app_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}