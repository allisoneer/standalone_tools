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
            
            // Desktop uses eager initialization
            app_state.initialize_transcription_blocking(init_data)?;
            
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(crate::register_app_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
