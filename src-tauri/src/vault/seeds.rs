//! Compile-time seed templates + plugin doc mirrors.
//!
//! This module bakes the contents of `src-tauri/src/templates/myspace/**/*.md`
//! and `src-tauri/src/plugins/active/*.md` into the binary via `include_str!`,
//! so the bootstrap functions in `vault::mod` can write them out at first
//! onboarding without depending on any runtime filesystem layout.
//!
//! ## Lifecycle (Q1 = B, locked 2026-04-28)
//!
//! | Static          | Lifecycle                           | Owner after first write |
//! |-----------------|-------------------------------------|-------------------------|
//! | `SEEDS`         | Write-once: skip if target exists   | User                    |
//! | `PLUGIN_DOCS`   | Mirror-overwrite on every launch    | App                     |
//! | `INDEXED_DIRS`  | INDEX.md regenerated every launch   | App                     |
//!
//! Adding a new template = add a row here + drop the .md under `templates/`.
//! Adding a new active plugin doc = add a row + drop the .md under
//! `plugins/active/`. There is no other place to register; the registry is
//! intentionally a single static array per concern.

/// Path-relative-to-vault → file content. Walked by
/// [`crate::vault::seed_user_templates`].
///
/// Order is irrelevant; the function creates parent directories as needed.
pub static SEEDS: &[(&str, &str)] = &[
    // Roles
    ("roles/junior-role.md",     include_str!("../templates/myspace/roles/junior-role.md")),
    ("roles/specialist-role.md", include_str!("../templates/myspace/roles/specialist-role.md")),

    // Skills (4: 2 alias-bound + 2 always-on/on-demand)
    ("skills/junior-skill.md",      include_str!("../templates/myspace/skills/junior-skill.md")),
    ("skills/specialist-skill.md",  include_str!("../templates/myspace/skills/specialist-skill.md")),
    ("skills/index-skill.md",       include_str!("../templates/myspace/skills/index-skill.md")),
    ("skills/tools-index-skill.md", include_str!("../templates/myspace/skills/tools-index-skill.md")),

    // Per-model config (Q1 = B). Mirrors the catalog in services/model_catalog.rs.
    ("models/lfm2.5-350m.md",          include_str!("../templates/myspace/models/lfm2.5-350m.md")),
    ("models/lfm2.5-1.2b-instruct.md", include_str!("../templates/myspace/models/lfm2.5-1.2b-instruct.md")),
    ("models/lfm2.5-1.2b-thinking.md", include_str!("../templates/myspace/models/lfm2.5-1.2b-thinking.md")),
    ("models/lfm2.5-vl-450m.md",       include_str!("../templates/myspace/models/lfm2.5-vl-450m.md")),
];

/// Filename → file content. Walked by [`crate::vault::mirror_plugin_docs`]
/// to refresh `myspace/plugin/<name>.md` on every launch from the
/// repo-side source of truth.
///
/// User edits to `myspace/plugin/*.md` are intentionally lost on next
/// launch — these files describe the **compiled-in** plugin set, not user
/// preferences.
pub static PLUGIN_DOCS: &[(&str, &str)] = &[
    ("autostart.md",  include_str!("../plugins/active/autostart.md")),
    ("clipboard.md",  include_str!("../plugins/active/clipboard.md")),
    ("dialog.md",     include_str!("../plugins/active/dialog.md")),
    ("fs.md",         include_str!("../plugins/active/fs.md")),
    ("haptics.md",    include_str!("../plugins/active/haptics.md")),
    ("leap-ai.md",    include_str!("../plugins/active/leap-ai.md")),
    ("llama-cpp.md",  include_str!("../plugins/active/llama-cpp.md")),
    ("log.md",        include_str!("../plugins/active/log.md")),
    ("os.md",         include_str!("../plugins/active/os.md")),
    ("shortcuts.md",  include_str!("../plugins/active/shortcuts.md")),
    ("sql.md",        include_str!("../plugins/active/sql.md")),
    ("stronghold.md", include_str!("../plugins/active/stronghold.md")),
];

/// Subdirs of `myspace/` that get an auto-regenerated `INDEX.md` written
/// each launch. The agent reads these to discover what's available without
/// every .md frontmatter being injected into the system prompt.
///
/// Order here = display order in the top-level `myspace/INDEX.md`.
///
/// Notes:
/// - `tools/` and `mcp/` start empty — they're populated by Phase E and
///   future MCP work; we still index them so the slot exists.
/// - `notes/` and `tasks/` are user-data folders; deliberately NOT indexed
///   here (would be huge and noisy).
pub static INDEXED_DIRS: &[(&str, &str)] = &[
    ("roles",  "Roles"),
    ("skills", "Skills"),
    ("models", "Models"),
    ("plugin", "Plugins"),
    ("tools",  "Tools"),
    ("mcp",    "MCP servers"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_array_covers_all_template_categories() {
        // Sanity: 2 roles + 4 skills + 4 models = 10
        assert_eq!(SEEDS.len(), 10, "SEEDS count drift — update this test if you added a template");
        // Spot-check: every path is sub-relative to vault root, no leading slash.
        for (rel, _) in SEEDS {
            assert!(!rel.starts_with('/'), "seed path must be relative: {}", rel);
            assert!(rel.ends_with(".md"), "seed must be .md: {}", rel);
        }
    }

    #[test]
    fn plugin_docs_array_matches_active_dir_count() {
        // 12 active plugin docs as of 2026-04-28 (autostart, clipboard, dialog,
        // fs, haptics, leap-ai, llama-cpp, log, os, shortcuts, sql, stronghold).
        assert_eq!(PLUGIN_DOCS.len(), 12, "PLUGIN_DOCS count drift — update if a new plugin was added");
        for (name, body) in PLUGIN_DOCS {
            assert!(!body.is_empty(), "embedded plugin doc {} is empty", name);
            assert!(name.ends_with(".md"));
            assert!(!name.contains('/'), "plugin doc filename must be flat: {}", name);
        }
    }

    #[test]
    fn indexed_dirs_match_reserved_subset() {
        // Every indexed dir MUST be in RESERVED_FOLDER_NAMES — otherwise we'd
        // try to walk a non-reserved dir and produce a surprise INDEX.md in a
        // user-renamed folder.
        for (subdir, _) in INDEXED_DIRS {
            assert!(
                crate::vault::RESERVED_FOLDER_NAMES.contains(subdir),
                "indexed dir {} not in RESERVED_FOLDER_NAMES",
                subdir
            );
        }
    }
}
