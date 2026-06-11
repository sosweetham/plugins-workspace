---
biometric: minor
biometric-js: minor
---

Add macOS support: `checkStatus` and `authenticate` now work on macOS via the LocalAuthentication framework (Touch ID, optional Apple Watch approval via the new `allowWatch` option, and login-password fallback via `allowDeviceCredential`).
