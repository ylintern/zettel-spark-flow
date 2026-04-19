//! Vibo runtime configuration.
//!
//! Feature flags gate execution paths without removing code.
//! Phase 0 disables encryption/biometric flows; Phase 2/3 will flip the flags.

pub mod features;

// Re-export for ergonomic usage in other modules (e.g. `use crate::config::FLAGS`).
// `get_feature_flags` is intentionally NOT re-exported — it's invoked only as a
// Tauri command via the full `config::features::get_feature_flags` path.
#[allow(unused_imports)]
pub use features::{FeatureFlags, FLAGS};
