// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod commands;
#[cfg(desktop)]
mod desktop;
mod error;
#[cfg(mobile)]
mod mobile;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::Biometric;
#[cfg(mobile)]
pub use mobile::Biometric;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the biometric APIs.
pub trait BiometricExt<R: Runtime> {
    fn biometric(&self) -> &Biometric<R>;
}

impl<R: Runtime, T: Manager<R>> crate::BiometricExt<R> for T {
    fn biometric(&self) -> &Biometric<R> {
        self.state::<Biometric<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = Builder::new("biometric");
    // the invoke handler must only be registered on desktop: on mobile,
    // commands handled in Rust never reach the native Swift/Kotlin plugin,
    // which implements `status` and `authenticate` itself
    #[cfg(desktop)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::status,
        commands::authenticate
    ]);
    builder
        .setup(|app, api| {
            #[cfg(mobile)]
            let biometric = mobile::init(app, api)?;
            #[cfg(desktop)]
            let biometric = desktop::init(app, api)?;
            app.manage(biometric);
            Ok(())
        })
        .build()
}
