// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod common;

mod audio;
mod commands;
mod settings;
mod state;
mod storage;
mod transcription;

#[cfg(target_os = "android")]
mod android_audio;
#[cfg(target_os = "linux")]
mod linux_audio;

use tauri::Manager;


fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
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
