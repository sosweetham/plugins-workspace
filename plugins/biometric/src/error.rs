// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// Authentication failed or is unavailable.
    ///
    /// `code` matches the error code strings used by the mobile implementations
    /// (e.g. `userCancel`, `biometryNotAvailable`).
    #[error("{message}")]
    Authentication { code: String, message: String },
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // matches the `{ message, code }` rejection shape produced by
            // the mobile implementations via `invoke.reject(message, code:)`
            Self::Authentication { code, message } => {
                use serde::ser::SerializeStruct;
                let mut s = serializer.serialize_struct("Error", 2)?;
                s.serialize_field("message", message)?;
                s.serialize_field("code", code)?;
                s.end()
            }
            _ => serializer.serialize_str(self.to_string().as_ref()),
        }
    }
}
