use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(mobile)]
pub mod mobile;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("audio")
        .setup(|app, api| {
            #[cfg(mobile)]
            {
                let audio_handle = mobile::init(app, api)?;
                app.manage(audio_handle); // Store handle in app state
            }
            
            // Suppress warning when not building for mobile
            #[cfg(not(mobile))]
            let _ = (app, api);
            
            Ok(())
        })
        .build()
}