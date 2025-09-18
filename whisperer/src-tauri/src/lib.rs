
// Re-export all modules needed for both desktop and mobile
mod audio;
mod audio_processor;
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
            
            // Clone AppHandle for async initialization
            let app_handle = app.handle().clone();
            
            // Manage state immediately
            app.manage(app_state);
            
            // Spawn async initialization if API key is present
            if let Some(api_key) = init_data.api_key 
                && !api_key.is_empty() {
                let base_url = init_data.base_url;
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::state::AppState::<tauri::Wry>::initialize_transcription_async(
                        app_handle,
                        api_key,
                        Some(base_url),
                    ).await {
                        eprintln!("Failed to initialize transcription service: {}", e);
                    }
                });
            }
            
            Ok(())
        })
        .invoke_handler(crate::register_app_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}