// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthOptions {
    /// Enables authentication using the device's password. This feature is available on Android, iOS and macOS.
    pub allow_device_credential: bool,
    /// Label for the Cancel button. This feature is available on Android, iOS and macOS.
    pub cancel_title: Option<String>,
    /// Specifies the text displayed on the fallback button if biometric authentication fails. This feature is available on iOS and macOS.
    pub fallback_title: Option<String>,
    /// Title indicating the purpose of biometric verification. This feature is available Android only.
    pub title: Option<String>,
    /// SubTitle providing contextual information of biometric verification. This feature is available Android only.
    pub subtitle: Option<String>,
    /// Specifies whether additional user confirmation is required, such as pressing a button after successful biometric authentication. This feature is available Android only.
    pub confirmation_required: Option<bool>,
    /// Allows the user to authenticate by approving on a nearby, paired and unlocked Apple Watch. This feature is available macOS only.
    ///
    /// Requires the "Apple Watch" option to be enabled in System Settings > Touch ID & Password
    /// ("Use Apple Watch to unlock your applications and your Mac").
    pub allow_watch: Option<bool>,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum BiometryType {
    None = 0,
    TouchID = 1,
    FaceID = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub is_available: bool,
    pub biometry_type: BiometryType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}
