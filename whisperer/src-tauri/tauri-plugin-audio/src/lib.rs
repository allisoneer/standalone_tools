use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(mobile)]
mod mobile;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("audio")
        .setup(|app, api| {
            #[cfg(mobile)]
            mobile::init(app, api)?;
            
            // Suppress warning when not building for mobile
            #[cfg(not(mobile))]
            let _ = (app, api);
            
            Ok(())
        })
        .build()
}