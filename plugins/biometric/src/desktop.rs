// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Biometric<R>> {
    Ok(Biometric(app.clone()))
}

/// Access to the biometric APIs.
pub struct Biometric<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Biometric<R> {
    pub fn status(&self) -> crate::Result<Status> {
        #[cfg(target_os = "macos")]
        return Ok(macos::status());
        #[cfg(not(target_os = "macos"))]
        Ok(Status {
            is_available: false,
            biometry_type: BiometryType::None,
            error: Some("Biometric authentication is not available on this platform".into()),
            error_code: Some("biometryNotAvailable".into()),
        })
    }

    /// Prompts the user for authentication.
    ///
    /// Blocks until the user completes or dismisses the system prompt,
    /// so it must not be called on the main thread.
    pub fn authenticate(&self, reason: String, options: AuthOptions) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        return macos::authenticate(&reason, &options);
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (reason, options);
            Err(crate::Error::Authentication {
                code: "biometryNotAvailable".into(),
                message: "Biometric authentication is not available on this platform".into(),
            })
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSError, NSString};
    use objc2_local_authentication::{LABiometryType, LAContext, LAError, LAPolicy};

    use crate::models::*;

    // matches the error code strings of the iOS implementation
    fn error_code(code: isize) -> &'static str {
        let error = LAError(code);
        if error == LAError::AppCancel {
            "appCancel"
        } else if error == LAError::AuthenticationFailed {
            "authenticationFailed"
        } else if error == LAError::InvalidContext {
            "invalidContext"
        } else if error == LAError::NotInteractive {
            "notInteractive"
        } else if error == LAError::PasscodeNotSet {
            "passcodeNotSet"
        } else if error == LAError::SystemCancel {
            "systemCancel"
        } else if error == LAError::UserCancel {
            "userCancel"
        } else if error == LAError::UserFallback {
            "userFallback"
        } else if error == LAError::BiometryLockout {
            "biometryLockout"
        } else if error == LAError::BiometryNotAvailable {
            "biometryNotAvailable"
        } else if error == LAError::BiometryNotEnrolled {
            "biometryNotEnrolled"
        } else if error == LAError::CompanionNotAvailable {
            // LAErrorWatchNotAvailable before the Companion rename
            "watchNotAvailable"
        } else {
            "authenticationFailed"
        }
    }

    fn check_policy(context: &LAContext, policy: LAPolicy) -> Result<(), (String, String)> {
        unsafe { context.canEvaluatePolicy_error(policy) }.map_err(|e| {
            (
                e.localizedDescription().to_string(),
                error_code(e.code()).to_string(),
            )
        })
    }

    pub fn status() -> Status {
        let context = unsafe { LAContext::new() };
        let result = check_policy(&context, LAPolicy::DeviceOwnerAuthenticationWithBiometrics);
        // biometryType is only valid after canEvaluatePolicy was called
        let biometry_type = match unsafe { context.biometryType() } {
            LABiometryType::TouchID => BiometryType::TouchID,
            LABiometryType::FaceID => BiometryType::FaceID,
            _ => BiometryType::None,
        };
        match result {
            Ok(()) => Status {
                is_available: true,
                biometry_type,
                error: None,
                error_code: None,
            },
            Err((error, error_code)) => Status {
                is_available: false,
                biometry_type,
                error: Some(error),
                error_code: Some(error_code),
            },
        }
    }

    pub fn authenticate(reason: &str, options: &AuthOptions) -> crate::Result<()> {
        let policy = if options.allow_device_credential {
            // also covers Touch ID, Apple Watch and the login password
            LAPolicy::DeviceOwnerAuthentication
        } else if options.allow_watch == Some(true) {
            // LAPolicyDeviceOwnerAuthenticationWithBiometricsOrWatch before the Companion rename
            LAPolicy::DeviceOwnerAuthenticationWithBiometricsOrCompanion
        } else {
            LAPolicy::DeviceOwnerAuthenticationWithBiometrics
        };

        let context = unsafe { LAContext::new() };
        if let Err((message, code)) = check_policy(&context, policy) {
            return Err(crate::Error::Authentication { code, message });
        }

        let cancel_title = options.cancel_title.as_deref().map(NSString::from_str);
        // parity with the iOS implementation: an empty fallback title with
        // device credentials allowed shows the default fallback button
        let fallback_title = match (&options.fallback_title, options.allow_device_credential) {
            (Some(title), true) if title.is_empty() => None,
            (title, _) => title.as_deref().map(NSString::from_str),
        };
        unsafe {
            context.setTouchIDAuthenticationAllowableReuseDuration(0.0);
            context.setLocalizedCancelTitle(cancel_title.as_deref());
            context.setLocalizedFallbackTitle(fallback_title.as_deref());
        }

        let (tx, rx) = std::sync::mpsc::channel::<Result<(), (String, String)>>();
        let reply = RcBlock::new(move |success: Bool, error: *mut NSError| {
            let result = if success.as_bool() {
                Ok(())
            } else if let Some(error) = unsafe { error.as_ref() } {
                Err((
                    error.localizedDescription().to_string(),
                    error_code(error.code()).to_string(),
                ))
            } else {
                Err((
                    "Unknown authentication error".to_string(),
                    "authenticationFailed".to_string(),
                ))
            };
            let _ = tx.send(result);
        });

        unsafe {
            context.evaluatePolicy_localizedReason_reply(
                policy,
                &NSString::from_str(reason),
                &reply,
            );
        }

        // `context` must be kept alive until the reply block runs,
        // otherwise the evaluation is canceled
        match rx.recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err((message, code))) => Err(crate::Error::Authentication { code, message }),
            Err(_) => Err(crate::Error::Authentication {
                code: "authenticationFailed".into(),
                message: "Authentication reply channel closed".into(),
            }),
        }
    }
}
