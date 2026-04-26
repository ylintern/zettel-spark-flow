> **STATUS (2026-04-26):** Historical research. Phase 0 + 0.7-A complete; current state lives in `Guidelines/source-of-truth/PHASE_0_COMPLETION.md`. Findings here informed the ship; left for traceability.

# Agent C: YAML & Testing Research

## Executive Summary

**Research Date:** 2026-04-15  
**Focus:** Phase 0 Filesystem-First Architecture  
**Status:** Ready for Implementation Decision

---

## 1. YAML Parser Comparison

### 1.1 Crate Overview

| Crate | Version | Pros | Cons | Recommendation |
|-------|---------|------|------|----------------|
| **serde_yaml** | 0.9.34+deprecated | Mature serde integration; familiar API; good error messages | **DEPRECATED** - No longer maintained; uses unsafe-libyaml C bindings | **NOT RECOMMENDED** for new projects |
| **gray_matter** | 0.3.2 | Purpose-built for frontmatter; separates content from metadata; supports YAML/TOML/JSON; clean API | Uses yaml-rust2 internally; less control over serialization format; 45% docs coverage | **RECOMMENDED** for frontmatter-specific use |
| **yaml-rust2** | 0.11.0 | Pure Rust YAML 1.2 compliant; 100% docs; active maintenance; no unsafe code | Manual frontmatter extraction needed; more verbose API | **RECOMMENDED** if manual control needed |
| **yaml-rust** | (legacy) | Original pure Rust parser | Unmaintained; superseded by yaml-rust2 | **DEPRECATED** |

### 1.2 Detailed Analysis

#### serde_yaml (DEPRECATED - DO NOT USE)
```rust
// Example usage
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct FrontMatter {
    title: String,
    id: String,
}

let fm: FrontMatter = serde_yaml::from_str(yaml)?;
```
**Why Avoid:** Officially deprecated as of March 2024. Maintenance ceased. Security risk for long-term project.

#### gray_matter (RECOMMENDED)
```rust
use gray_matter::{Matter, engine::YAML};
use serde::Deserialize;

#[derive(Deserialize)]
struct FrontMatter {
    title: String,
    id: String,
}

let matter = Matter::<YAML>::new();
let result = matter.parse::<FrontMatter>(markdown)?;
// result.content = body
// result.data = FrontMatter struct
```
**Key Benefits:**
- Automatically handles `---` delimiter detection
- Separates frontmatter from body content
- Returns clean body without frontmatter
- Supports excerpt extraction (content before `---` in body)

#### yaml-rust2 (ALTERNATIVE)
```rust
use yaml_rust2::{YamlLoader, YamlEmitter};

let docs = YamlLoader::load_from_str(frontmatter_yaml)?;
let doc = &docs[0];
let title = doc["title"].as_str().unwrap();

// Manual frontmatter extraction needed:
// 1. Split on "---\n"
// 2. Parse YAML section
// 3. Remainder is body
```
**Key Benefits:**
- Full YAML 1.2 compliance
- Pure Rust (no C dependencies)
- Fine-grained control over parsing
- Active development

### 1.3 Frontmatter Requirements

Current format in `vault/mod.rs`:
```yaml
---
id: "..."
type: note|task
title: "..."
column: "..."
position: 0
is_encrypted: false
created: "2026-04-15T..."
modified: "2026-04-15T..."
folder: null
  # or: "folder-name"
tags:
  - tag1
  - tag2
  # or: []
---

# Body content here
```

### 1.4 Current Implementation Issues

Looking at `vault/mod.rs`:

1. **Manual YAML generation** (lines 116-186):
   - Uses `yaml_string()` which just wraps with `format!("{value:?}")`
   - This is Rust debug formatting, NOT proper YAML escaping
   - Risk: Special characters in titles/content will break YAML

2. **Manual frontmatter extraction** (lines 188-196):
   ```rust
   fn extract_body(markdown: &str) -> String {
       if let Some(stripped) = markdown.strip_prefix("---\n") {
           if let Some(idx) = stripped.find("\n---\n") {
               return stripped[(idx + 5)..].trim_start_matches('\n').to_string();
           }
       }
       markdown.to_string()
   }
   ```
   - No YAML parsing at all
   - Will fail if frontmatter has `---` in content
   - No validation of YAML structure

### 1.5 Recommended Implementation

**Primary:** Use `gray_matter` crate

Cargo.toml:
```toml
[dependencies]
gray_matter = { version = "0.3", features = ["yaml"] }
serde = { version = "1.0", features = ["derive"] }
```

Implementation approach:
```rust
use gray_matter::{Matter, engine::YAML};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteFrontmatter {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub note_type: String,
    pub column: String,
    pub position: i64,
    #[serde(rename = "is_encrypted")]
    pub is_encrypted: bool,
    pub created: String,
    pub modified: String,
    pub folder: Option<String>,
    pub tags: Vec<String>,
}

pub fn parse_note(content: &str) -> Result<(NoteFrontmatter, String), Error> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse::<NoteFrontmatter>(content)?;
    
    let frontmatter = result.data.ok_or_else(|| Error::NoFrontmatter)?;
    let body = result.content;
    
    Ok((frontmatter, body))
}

pub fn render_note(frontmatter: &NoteFrontmatter, body: &str) -> Result<String, Error> {
    let yaml = serde_yaml::to_string(frontmatter)?; // Note: or use gray_matter's serialize
    format!("---\n{}---\n\n{}", yaml, body)
}
```

**Important:** Since `serde_yaml` is deprecated, for serialization we should use:
1. Option A: Use `gray_matter` + manual YAML serialization via `yaml-rust2`
2. Option B: Use `yaml-rust2` for both parsing and emitting

Recommended Cargo.toml:
```toml
[dependencies]
gray_matter = { version = "0.3", features = ["yaml"] }
yaml-rust2 = "0.11"
serde = { version = "1.0", features = ["derive"] }
```

---

## 2. Obsidian Compatibility Research

### 2.1 What YAML Format Does Obsidian Expect?

**Obsidian Properties (Frontmatter) Documentation Summary:**

1. **Delimiters:** Must use exactly `---` on its own line
   - Opening: `---` at start of file (or after BOM)
   - Closing: `---` on its own line
   - No trailing spaces allowed on delimiter lines

2. **Supported Formats:**
   - YAML (primary and recommended)
   - JSON (recognized but not preferred)
   - TOML (via plugin)

3. **Field Types Obsidian Recognizes:**
   - **Text:** Any string value
   - **List:** YAML arrays (`tags: ["a", "b"]` or `tags:\n  - a\n  - b`)
   - **Number:** Integer or float
   - **Checkbox:** `true`/`false` (displayed as checkbox in UI)
   - **Date:** ISO 8601 format (`2026-04-15`)
   - **DateTime:** ISO 8601 with time

4. **Native Fields:**
   - `title` - Displayed in file explorer, graph view
   - `tags` - Integrated with tag pane, clickable
   - `aliases` - Alternative titles for linking
   - `cssclasses` - Apply CSS classes

5. **Unknown Fields:**
   - Obsidian preserves but ignores unknown fields
   - Shown in Properties view as "Other properties"
   - Not editable via UI but preserved on save

### 2.2 Field Compatibility Matrix

| ViBo Field | Obsidian Behavior | Notes |
|------------|-------------------|-------|
| `title` | **Native** - Displayed prominently | Must be string |
| `type` | Unknown - Ignored but preserved | ViBo-specific |
| `id` | Unknown - Ignored but preserved | ViBo-specific |
| `column` | Unknown - Ignored but preserved | ViBo-specific |
| `position` | Unknown - Ignored but preserved | ViBo-specific |
| `is_encrypted` | Unknown - Shows as boolean | Consider renaming to `encrypted` for clarity |
| `created` | **Date** - Recognized as date if ISO8601 | Good for "Created" sorting |
| `modified` | **Date** - Recognized as date if ISO8601 | Good for "Modified" sorting |
| `folder` | Unknown - Ignored | ViBo-specific |
| `tags` | **Native** - Fully integrated | Array format required |

### 2.3 Critical Compatibility Requirements

1. **Quote Handling:**
   - Titles with quotes: `title: "Chapter 1: \"The Beginning\""`
   - Must properly escape quotes in YAML

2. **Multiline Content:**
   - Frontmatter should not contain multiline strings
   - Body is after second `---`

3. **Special Characters:**
   - `#` at start of line is comment in YAML - must quote
   - `:` in values must be quoted if followed by space
   - `-` at start is list item

4. **UTF-8:**
   - Obsidian expects UTF-8 encoding
   - BOM optional but supported

### 2.4 Testing Compatibility Without Obsidian

Since we cannot programmatically run Obsidian in tests, we validate compatibility by:

1. **YAML Compliance:** Pass through strict YAML 1.2 parser
2. **Format Verification:** Check exact `---` delimiters
3. **Field Types:** Match Obsidian's documented types
4. **Reference Samples:** Test against known Obsidian-compatible files
5. **Community Validators:** Use tools like `obsidian-export` or `obsidian-dataview` test suites

---

## 3. Testing Strategy

### 3.1 Unit Tests

#### YAML Parse Roundtrip
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_roundtrip() {
        let original = NoteFrontmatter {
            id: "550e8400-e29b-41d4".to_string(),
            title: "Test Note".to_string(),
            note_type: "note".to_string(),
            column: "default".to_string(),
            position: 0,
            is_encrypted: false,
            created: "2026-04-15T10:00:00Z".to_string(),
            modified: "2026-04-15T10:00:00Z".to_string(),
            folder: None,
            tags: vec!["test".to_string()],
        };
        
        let body = "# Test Content\n\nHello world";
        let markdown = render_note(&original, body).unwrap();
        let (parsed_fm, parsed_body) = parse_note(&markdown).unwrap();
        
        assert_eq!(original.id, parsed_fm.id);
        assert_eq!(original.title, parsed_fm.title);
        assert_eq!(body, parsed_body);
    }
}
```

#### Frontmatter Extraction Edge Cases
```rust
#[test]
fn test_frontmatter_with_special_chars() {
    let cases = vec![
        ("Title with \"quotes\"", "Title with \"quotes\""),
        ("Title with: colon", "Title with: colon"),
        ("Title with # hash", "Title with # hash"),
        ("Multi\nLine", "Multi Line"), // Should sanitize or reject
    ];
    
    for (input, expected) in cases {
        let fm = NoteFrontmatter {
            title: input.to_string(),
            // ...
        };
        let rendered = render_note(&fm, "").unwrap();
        let (parsed, _) = parse_note(&rendered).unwrap();
        assert_eq!(parsed.title, expected);
    }
}
```

#### Body Extraction
```rust
#[test]
fn test_body_extraction() {
    let cases = vec![
        // Standard case
        ("---\ntitle: Test\n---\n\nBody", "Body"),
        // No trailing newline
        ("---\ntitle: Test\n---\nBody", "Body"),
        // Body with --- in code block
        ("---\ntitle: Test\n---\n\n```\n---\n```", "```\n---\n```"),
        // Empty body
        ("---\ntitle: Test\n---\n", ""),
    ];
    
    for (input, expected) in cases {
        let (_, body) = parse_note(input).unwrap();
        assert_eq!(body.trim(), expected.trim());
    }
}
```

### 3.2 Integration Tests

#### Create -> Read -> Update -> Delete Cycle
```rust
#[tokio::test]
async fn test_crud_cycle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_dir = temp_dir.path();
    
    // Create
    let note = create_note(vault_dir, "Test Note", None).await.unwrap();
    assert!(vault_dir.join("notes").join(format!("{}.md", note.id)).exists());
    
    // Read
    let content = read_note(vault_dir, &note.id).await.unwrap();
    assert!(content.contains("Test Note"));
    
    // Update
    update_note(vault_dir, &note.id, "Updated content").await.unwrap();
    let updated = read_note(vault_dir, &note.id).await.unwrap();
    assert!(updated.contains("Updated content"));
    
    // Delete
    delete_note(vault_dir, &note.id).await.unwrap();
    assert!(!vault_dir.join("notes").join(format!("{}.md", note.id)).exists());
}
```

#### Obsidian Compatibility Fixtures
```rust
#[test]
fn test_obsidian_compatibility() {
    // Test against manually verified Obsidian-compatible files
    let fixtures = vec![
        "tests/fixtures/obsidian-note.md",
        "tests/fixtures/obsidian-task.md",
    ];
    
    for fixture in fixtures {
        let content = fs::read_to_string(fixture).unwrap();
        let (fm, body) = parse_note(&content).unwrap();
        
        // Verify structure
        assert!(!fm.id.is_empty());
        assert!(!fm.title.is_empty());
        assert!(fm.note_type == "note" || fm.note_type == "task");
        
        // Verify Obsidian compatibility markers
        let yaml_str = serde_yaml::to_string(&fm).unwrap();
        assert!(!yaml_str.contains('\t')); // No tabs in YAML
    }
}
```

### 3.3 Test Fixtures Structure

```
tests/fixtures/
├── obsidian-note.md              # Manually created in Obsidian
├── obsidian-task.md              # Manually created in Obsidian
├── minimal-note.md               # Minimal valid frontmatter
├── edge-cases/
│   ├── special-chars.md          # Quotes, colons, hashes
│   ├── unicode.md                # Emoji, CJK, RTL text
│   ├── large-content.md          # 1MB+ body
│   ├── empty-tags.md             # tags: []
│   ├── null-folder.md            # folder: null
│   ├── multiline-title.md        # Edge case: should sanitize
│   ├── windows-line-endings.md   # CRLF vs LF
│   └── bom-utf8.md               # Byte order marker
└── invalid/
    ├── missing-delimiter.md      # No closing ---
    ├── invalid-yaml.md           # Malformed YAML
    ├── duplicate-keys.md         # YAML duplicate keys
    └── body-before-frontmatter.md
```

### 3.4 Cross-Platform Path Handling

```rust
#[test]
fn test_cross_platform_paths() {
    let test_cases = vec![
        ("notes/test-note.md", Path::new("notes").join("test-note.md")),
        ("notes\\test-note.md", Path::new("notes").join("test-note.md")), // Windows
        ("notes/sub/folder/note.md", Path::new("notes").join("sub").join("folder").join("note.md")),
    ];
    
    for (input, expected) in test_cases {
        let path = normalize_path(input);
        assert_eq!(path, expected);
    }
}
```

### 3.5 Mocking for Filesystem Tests

Since Phase 0 is filesystem-first, we need reliable ways to test file operations:

```rust
// Using tempfile crate for isolated test directories
use tempfile::TempDir;

fn setup_test_vault() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    ensure_vault_dirs(dir.path()).unwrap();
    dir
}

// Tests run in parallel with unique temp directories
#[tokio::test]
async fn parallel_safe_test() {
    let vault = setup_test_vault();
    // ... test operations ...
    // Directory automatically cleaned up on drop
}
```

---

## 4. Implementation Recommendations

### 4.1 Crate Selection

**Primary Choice: `gray_matter` + `yaml-rust2`**

```toml
[dependencies]
gray_matter = { version = "0.3", features = ["yaml"] }
yaml-rust2 = "0.11"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

**Rationale:**
1. Purpose-built for frontmatter extraction
2. Properly handles delimiter separation
3. Uses yaml-rust2 (active, pure Rust)
4. Clean API for our use case
5. Supports future TOML/JSON if needed

### 4.2 Migration Strategy

**Current State (Manual YAML):**
```rust
fn yaml_string(value: &str) -> String {
    format!("{value:?}")  // BROKEN - debug formatting
}
```

**Target State (gray_matter):**
```rust
use gray_matter::{Matter, engine::YAML};

pub struct NoteParser {
    matter: Matter<YAML>,
}

impl NoteParser {
    pub fn new() -> Self {
        Self { matter: Matter::new() }
    }
    
    pub fn parse(&self, content: &str) -> Result<ParsedNote> {
        let result = self.matter.parse::<NoteFrontmatter>(content)?;
        Ok(ParsedNote {
            frontmatter: result.data?,
            body: result.content,
            excerpt: result.excerpt,
        })
    }
}
```

### 4.3 Testing Approach

1. **Phase 0 (Current):**
   - Unit tests for parse/render roundtrip
   - Integration tests for CRUD operations
   - Manual Obsidian compatibility verification

2. **Phase 1+:**
   - Property-based testing with `proptest`
   - Snapshot testing for output format
   - CI tests with actual Obsidian Docker container (if available)

### 4.4 Risk Mitigation

| Risk | Mitigation |
|------|------------|
| YAML parsing errors | Wrap in `anyhow` with context; validate before writing |
| Obsidian format drift | Maintain fixture files; manual verification each release |
| Special character injection | Sanitize all user input; escape test matrix |
| Encoding issues | Force UTF-8; strip BOM on read; validate with `encoding_rs` |
| Concurrent writes | File locking via `fs2` or atomic writes (temp + rename) |

---

## 5. Action Items

### 5.1 Immediate (Phase 0)

- [ ] Add to Cargo.toml: `gray_matter = { version = "0.3", features = ["yaml"] }`
- [ ] Add to Cargo.toml: `yaml-rust2 = "0.11"`
- [ ] Create `NoteFrontmatter` struct with serde derives
- [ ] Refactor `vault/mod.rs` `render_markdown()` to use proper YAML emission
- [ ] Refactor `vault/mod.rs` `extract_body()` to use gray_matter parser
- [ ] Create test fixtures directory structure
- [ ] Write unit tests for YAML roundtrip
- [ ] Write integration tests for CRUD operations
- [ ] Manual test: Create note, open in Obsidian, verify

### 5.2 Short-term (Phase 0 Complete)

- [ ] Property-based tests for edge cases
- [ ] CI job for fixture validation
- [ ] Document Obsidian compatibility test procedure
- [ ] Create test data generator for stress testing

### 5.3 Long-term (Phase 1+)

- [ ] Consider `quickcheck` or `proptest` for generative testing
- [ ] File watcher tests (when watcher implemented)
- [ ] Migration tests from Phase 0 format
- [ ] Performance benchmarks for large vaults

---

## 6. Appendix

### 6.1 References

- **serde_yaml deprecation:** https://github.com/dtolnay/serde-yaml/commit/09b1968
- **gray_matter docs:** https://docs.rs/gray_matter/
- **yaml-rust2 docs:** https://docs.rs/yaml-rust2/
- **Obsidian Properties:** https://help.obsidian.md/Editing+and+formatting/Properties
- **YAML 1.2 Spec:** https://yaml.org/spec/1.2.2/

### 6.2 Example Test Fixture: obsidian-note.md

```markdown
---
id: "550e8400-e29b-41d4-a716-446655440000"
type: note
title: "Sample Obsidian Note"
column: "default"
position: 0
is_encrypted: false
created: "2026-04-15T10:00:00Z"
modified: "2026-04-15T10:30:00Z"
folder: null
tags:
  - sample
  - obsidian-compatible
---

# Sample Obsidian Note

This is a test note that should be fully compatible with Obsidian.

## Features

- Standard YAML frontmatter
- Compatible tags
- Regular markdown body

## Code Block Test

```rust
// This should not interfere with frontmatter parsing
fn main() {
    println!("Hello, Obsidian!");
}
```

## Links

- [[Another Note]]
- [External link](https://example.com)
```

### 6.3 Example Test Fixture: special-chars.md

```markdown
---
id: "test-special-chars"
type: note
title: "Note with \"quotes\" and: colons"
column: "default"
position: 0
is_encrypted: false
created: "2026-04-15T10:00:00Z"
modified: "2026-04-15T10:00:00Z"
folder: null
tags:
  - "tag-with-dash"
  - "#not-a-comment"
---

# Special Characters Test

This note tests YAML escaping of special characters.
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-04-15  
**Author:** Agent C (Research)
