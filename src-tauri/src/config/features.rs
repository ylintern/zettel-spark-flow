//! Phase 0 Runtime Feature Flags
//!
//! Gates execution paths for features that are IMPLEMENTED but DORMANT in Phase 0.
//! Code for these features remains fully present — flipping the flag re-activates
//! the original behavior end-to-end without code resurrection.
//!
//! See: Guidelines/current-phase/PHASE_0_STRATEGY.md
//!
//! ## Phase Schedule
//! - Phase 0 (current): encryption=false, biometric=false
//! - Phase 2/3: encryption=true, biometric=true
//! - Phase 10: ios_enabled=true
//!
//! ## Usage
//! ```rust,ignore
//! use crate::config::FLAGS;
//!
//! if FLAGS.encryption_enabled {
//!     security.encrypt_note_content(body)
//! } else {
//!     plaintext_body.to_string()
//! }
//! ```

use serde::Serialize;

/// Compile-time feature flags. Mutated only by editing this file + recompile.
/// Runtime-configurable flags (e.g. user preferences) belong elsewhere.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FeatureFlags {
    /// Gate AES-256-GCM note encryption + Stronghold-derived key unlock.
    /// When `false`: notes are written plaintext; `is_encrypted` field defaults to `false`;
    /// read path skips decrypt; UI hides lock icon + skips LockScreen mount.
    /// When `true`: full Phase 1+ encryption behavior restored.
    pub encryption_enabled: bool,

    /// Gate biometric unlock (Face ID / Android fingerprint).
    /// When `false`: Settings hides biometric section; no biometric commands invoked.
    /// When `true`: original biometric code path re-enabled.
    pub biometric_enabled: bool,

    /// Gate iOS-specific setup (iCloud Drive, Files app sharing).
    /// Phase 0: Desktop + Android only. iOS deferred to Phase 10.
    pub ios_enabled: bool,
}

impl FeatureFlags {
    /// Phase 0 defaults: encryption and biometric dormant, iOS deferred.
    pub const fn phase_0() -> Self {
        Self {
            encryption_enabled: false,
            biometric_enabled: false,
            ios_enabled: false,
        }
    }
}

/// Global feature flag instance. Phase 0 configuration.
pub const FLAGS: FeatureFlags = FeatureFlags::phase_0();

/// Tauri command: expose feature flags to the TSX frontend.
///
/// TSX uses this on mount to decide whether to render lock icons,
/// mount LockScreen, show Settings biometric section, etc.
#[tauri::command]
pub fn get_feature_flags() -> FeatureFlags {
    FLAGS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_0_defaults_are_dormant() {
        let flags = FeatureFlags::phase_0();
        assert!(!flags.encryption_enabled);
        assert!(!flags.biometric_enabled);
        assert!(!flags.ios_enabled);
    }

    #[test]
    fn const_flags_match_phase_0() {
        assert_eq!(FLAGS.encryption_enabled, false);
        assert_eq!(FLAGS.biometric_enabled, false);
        assert_eq!(FLAGS.ios_enabled, false);
    }

    #[test]
    fn flags_serialize_to_json() {
        let flags = FeatureFlags::phase_0();
        let json = serde_json::to_string(&flags).unwrap();
        assert!(json.contains("encryption_enabled"));
        assert!(json.contains("false"));
    }
}
