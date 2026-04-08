use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

use crate::state::AppState;

/// Mobile Biometric Unlock State
///
/// This module is intentionally conservative: it only enables biometric unlock
/// when a real hardware-backed key release path exists.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiometricConfig {
    pub enabled: bool,
    pub device_supports_biometric: bool,
    pub hardware_backed_release: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCapabilities {
    pub platform: &'static str,
    pub is_desktop: bool,
    pub is_mobile: bool,
    pub has_secure_enclave: bool,
    pub has_touch_id: bool,
    pub has_face_id: bool,
    pub supports_biometric_prompt: bool,
    pub can_offer_biometrics: bool,
    pub supports_pin: bool,
    pub supports_passphrase: bool,
}

pub struct BiometricState {
    capabilities: DeviceCapabilities,
    config: Mutex<BiometricConfig>,
}

impl BiometricState {
    pub fn new() -> Self {
        let capabilities = detect_device_capabilities();
        Self {
            capabilities: capabilities.clone(),
            config: Mutex::new(BiometricConfig {
                enabled: false,
                device_supports_biometric: capabilities.supports_biometric_prompt,
                hardware_backed_release: capabilities.can_offer_biometrics,
            }),
        }
    }

    pub fn capabilities(&self) -> DeviceCapabilities {
        self.capabilities.clone()
    }

    pub fn is_biometric_available(&self) -> bool {
        let cfg = self.config.lock().unwrap();
        cfg.device_supports_biometric && self.capabilities.can_offer_biometrics
    }

    pub fn is_biometric_enabled(&self) -> bool {
        let cfg = self.config.lock().unwrap();
        cfg.enabled && cfg.hardware_backed_release
    }

    pub fn enable_biometric(&self) -> Result<(), String> {
        let mut cfg = self.config.lock().unwrap();

        if !cfg.device_supports_biometric {
            return Err("This device does not support biometric authentication".to_string());
        }

        if !cfg.hardware_backed_release {
            return Err(
                "Hardware-backed biometric vault unlock is not implemented in this build yet"
                    .to_string(),
            );
        }

        cfg.enabled = true;
        Ok(())
    }

    pub fn disable_biometric(&self) -> Result<(), String> {
        let mut cfg = self.config.lock().unwrap();
        cfg.enabled = false;
        Ok(())
    }

    pub fn reset(&self) {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.enabled = false;
        }
    }

    pub fn can_release_vault_key(&self) -> bool {
        let cfg = self.config.lock().unwrap();
        cfg.device_supports_biometric && cfg.hardware_backed_release && cfg.enabled
    }

    pub fn audit_summary(&self) -> &'static str {
        if self.capabilities.is_mobile {
            "Biometric UI path exists, but hardware-backed key release is not implemented yet"
        } else {
            "Biometric vault unlock is mobile-only and unavailable on desktop builds"
        }
    }

    #[cfg(test)]
    pub fn set_hardware_backed_release_available(&self, available: bool) {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.hardware_backed_release = available;
            if !available {
                cfg.enabled = false;
            }
        }
    }

    pub fn get_config(&self) -> BiometricConfig {
        self.config.lock().unwrap().clone()
    }
}

fn detect_device_capabilities() -> DeviceCapabilities {
    #[cfg(target_os = "ios")]
    {
        return DeviceCapabilities {
            platform: "ios",
            is_desktop: false,
            is_mobile: true,
            has_secure_enclave: true,
            has_touch_id: true,
            has_face_id: true,
            supports_biometric_prompt: true,
            can_offer_biometrics: false,
            supports_pin: true,
            supports_passphrase: false,
        };
    }

    #[cfg(target_os = "android")]
    {
        return DeviceCapabilities {
            platform: "android",
            is_desktop: false,
            is_mobile: true,
            has_secure_enclave: false,
            has_touch_id: true,
            has_face_id: false,
            supports_biometric_prompt: true,
            can_offer_biometrics: false,
            supports_pin: true,
            supports_passphrase: false,
        };
    }

    #[cfg(target_os = "macos")]
    {
        return DeviceCapabilities {
            platform: "macos",
            is_desktop: true,
            is_mobile: false,
            has_secure_enclave: false,
            has_touch_id: false,
            has_face_id: false,
            supports_biometric_prompt: false,
            can_offer_biometrics: false,
            supports_pin: true,
            supports_passphrase: true,
        };
    }

    #[cfg(target_os = "windows")]
    {
        return DeviceCapabilities {
            platform: "windows",
            is_desktop: true,
            is_mobile: false,
            has_secure_enclave: false,
            has_touch_id: false,
            has_face_id: false,
            supports_biometric_prompt: false,
            can_offer_biometrics: false,
            supports_pin: true,
            supports_passphrase: true,
        };
    }

    #[cfg(target_os = "linux")]
    {
        return DeviceCapabilities {
            platform: "linux",
            is_desktop: true,
            is_mobile: false,
            has_secure_enclave: false,
            has_touch_id: false,
            has_face_id: false,
            supports_biometric_prompt: false,
            can_offer_biometrics: false,
            supports_pin: true,
            supports_passphrase: true,
        };
    }

    #[allow(unreachable_code)]
    DeviceCapabilities {
        platform: "unknown",
        is_desktop: true,
        is_mobile: false,
        has_secure_enclave: false,
        has_touch_id: false,
        has_face_id: false,
        supports_biometric_prompt: false,
        can_offer_biometrics: false,
        supports_pin: true,
        supports_passphrase: true,
    }
}

/// Command: Check if device supports biometric authentication
///
/// Returns true on iOS/Android, false on desktop
#[tauri::command]
pub async fn is_biometric_available(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.biometric.is_biometric_available())
}

#[tauri::command]
pub async fn get_device_capabilities(
    state: State<'_, AppState>,
) -> Result<DeviceCapabilities, String> {
    Ok(state.biometric.capabilities())
}

/// Command: Check if user has enabled biometric unlock
#[tauri::command]
pub async fn is_biometric_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.biometric.is_biometric_enabled())
}

/// Command: Enable biometric unlock (iOS/Android only)
///
/// This hardening pass refuses to enable the feature unless a hardware-backed
/// key release path has been wired for the current build.
#[tauri::command]
pub async fn enable_biometric_unlock(
    _encrypted_passphrase: String,
    state: State<'_, AppState>,
) -> Result<BiometricConfig, String> {
    state.biometric.enable_biometric()?;
    Ok(state.biometric.get_config())
}

/// Command: Disable biometric unlock
#[tauri::command]
pub async fn disable_biometric_unlock(
    state: State<'_, AppState>,
) -> Result<BiometricConfig, String> {
    state.biometric.disable_biometric()?;
    Ok(state.biometric.get_config())
}

/// Command: Trigger biometric verification (native OS dialog)
///
/// On iOS: Shows FaceID or Fallback to Passcode UI
/// On Android: Shows Fingerprint or Biometric dialog
///
/// On desktop: Returns error (not supported)
///
/// This command delegates to native code via Tauri plugin system.
/// Plugin: tauri-plugin-biometric (to be added)
///
/// Returns: `true` if biometric verification succeeded, `false` if cancelled/failed
#[tauri::command]
pub async fn verify_biometric_and_unlock(state: State<'_, AppState>) -> Result<bool, String> {
    // Check if this is a mobile platform
    if !cfg!(any(target_os = "android", target_os = "ios")) {
        return Err("Biometric authentication is only supported on mobile".to_string());
    }

    // Check if biometric is enabled
    if !state.biometric.is_biometric_enabled() {
        return Err("Biometric unlock not configured".to_string());
    }

    if !state.biometric.can_release_vault_key() {
        return Err(state.biometric.audit_summary().to_string());
    }

    Ok(false)
}

/// Fallback: If biometric fails, allow user to enter passphrase manually
///
/// This command is called when:
/// - User taps "Try passphrase instead" on lock screen
/// - Biometric verification fails
/// - Device does not support biometric
#[tauri::command]
pub async fn fallback_passphrase_unlock(
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Delegate to standard passphrase unlock
    state
        .security
        .unlock(&passphrase)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_state_creation() {
        let state = BiometricState::new();

        #[cfg(any(target_os = "android", target_os = "ios"))]
        assert!(state.is_biometric_available());

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        assert!(!state.is_biometric_available());
    }

    #[test]
    fn test_enable_biometric() {
        let state = BiometricState::new();

        // On mobile, should succeed
        if state.is_biometric_available() {
            state.set_hardware_backed_release_available(true);
            let result = state.enable_biometric();
            assert!(result.is_ok());
            assert!(state.is_biometric_enabled());
        }
    }

    #[test]
    fn test_disable_biometric() {
        let state = BiometricState::new();

        if state.is_biometric_available() {
            state.set_hardware_backed_release_available(true);
            state.enable_biometric().unwrap();
            state.disable_biometric().unwrap();
            assert!(!state.is_biometric_enabled());
        }
    }
}
