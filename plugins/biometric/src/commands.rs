// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, AppHandle, Runtime};

use crate::{AuthOptions, BiometricExt, Result, Status};

#[command]
pub(crate) async fn status<R: Runtime>(app: AppHandle<R>) -> Result<Status> {
    app.biometric().status()
}

// the JS API sends the options flattened alongside `reason`,
// so they are received as individual arguments here
#[command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn authenticate<R: Runtime>(
    app: AppHandle<R>,
    reason: String,
    allow_device_credential: Option<bool>,
    cancel_title: Option<String>,
    fallback_title: Option<String>,
    title: Option<String>,
    subtitle: Option<String>,
    confirmation_required: Option<bool>,
    allow_watch: Option<bool>,
) -> Result<()> {
    let options = AuthOptions {
        allow_device_credential: allow_device_credential.unwrap_or(false),
        cancel_title,
        fallback_title,
        title,
        subtitle,
        confirmation_required,
        allow_watch,
    };
    // authenticate() blocks until the user completes or dismisses the
    // system prompt, so it must run off the main thread
    tauri::async_runtime::spawn_blocking(move || app.biometric().authenticate(reason, options))
        .await
        .map_err(|e| crate::Error::Authentication {
            code: "authenticationFailed".into(),
            message: e.to_string(),
        })?
}
