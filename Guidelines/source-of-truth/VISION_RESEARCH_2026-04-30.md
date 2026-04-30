# Vision Research — mmproj-aware multimodal inference for Vibo

> **Doc role:** Source-of-truth research note feeding Phase 0.7-B Phase F2.
> **Author:** Research agent (read-only pass)
> **Date:** 2026-04-30
> **Filename note:** the catalog (`src-tauri/src/services/model_catalog.rs`)
> still references the older `VISION_RESEARCH_2026-04-28.md` filename. That
> reference will be updated when Phase F2 picks this up — out of scope here.
> **Web access:** WebFetch / WebSearch were unavailable during this pass, so
> external version/changelog claims below are flagged with confidence levels
> and concrete unknowns are listed in §5. The integration plan does not
> depend on any of those unknowns being resolved a particular way; it only
> branches on a binary "mmproj API exposed in 0.1.x?" question.

---

## 1. Question being answered

How does Vibo wire **image input** into its existing in-process llama.cpp
chat path (`services/llama.rs`) so that the already-registered but disabled
`lfm2.5-vl-450m` (alias `inspector`) can describe / answer about PNG bytes
the user attaches in chat — **without** spawning subprocesses, **without**
adding a new Tauri plugin, and **without** breaking the dedupe between our
direct `llama-cpp-2 = "0.1"` dep and the copy `tauri-plugin-leap-ai = "0.1.1"`
pulls in transitively. We need a concrete crate choice, an integration plan
with file-level signatures, and an effort estimate sized for one engineer.

---

## 2. Crate landscape

| crate                  | latest version (training-cutoff knowledge, Jan 2026) | mmproj / mtmd support                                                                                              | image-encode API (in Rust)                                                          | maintained?                                | license      | docs URL                                            | fits 0.1.x line? |
| ---------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------- | ------------------------------------------ | ------------ | --------------------------------------------------- | ---------------- |
| `llama-cpp-2`          | 0.1.x line (utilityai/llama-cpp-rs)                  | **Partial / evolving.** 0.1.x historically shipped `clip`-based mmproj via the legacy `llama_clip_*` C symbols; whether `mtmd_*` (the post-mid-2025 upstream replacement) is wrapped in 0.1.x is **UNKNOWN — see §5**. | `LlamaModel::load_from_file` for the LM; mmproj loaded via a separate `clip`/`mtmd` ctx; image embeddings spliced into `LlamaBatch` | yes — same repo as our text path           | MIT / Apache | <https://docs.rs/llama-cpp-2>                       | **YES (already pinned)** |
| `llama_cpp`            | 0.10.x (edgenai/llama_cpp-rs, **underscore**)        | Limited — historically text-only LLaMA bindings; no first-class mmproj wrapper at last public reading              | n/a in tree                                                                         | semi-active, slower than utilityai's fork  | MIT          | <https://docs.rs/llama_cpp>                         | NO — different upstream, would replace our text path |
| `candle-transformers`  | 0.8.x (huggingface/candle)                           | **No GGUF mmproj.** Has its own LLaVA/SigLIP/ViT models in pure Rust + safetensors; doesn't read `mmproj-*.gguf`   | `candle_transformers::models::llava::*` — image -> tensor via candle's ViT encoder  | very active (HF)                           | Apache-2.0   | <https://docs.rs/candle-transformers>               | NO — would shadow llama-cpp-2 entirely; doubles binary |
| `mistral.rs` / `mistralrs` | 0.x active (EricLBuehler/mistral.rs)             | Yes — supports vision models (LLaVA, Phi-3-V, Idefics, Pixtral) with its own image preprocessing; uses safetensors / its own GGUF subset | `mistralrs::VisionModelBuilder`, `Request::with_image_bytes`-style API              | very active                                | MIT          | <https://docs.rs/mistralrs> / <https://github.com/EricLBuehler/mistral.rs> | NO — pulls a second inference engine alongside llama.cpp |

### Notes on the table

- **`llama-cpp-2` is the only candidate that fits the architecture rules without a parallel inference engine.** Every other row would either replace the text path we just stabilised in T10/T12 or ship a second backend next to it (binary-size + dedupe nightmare).
- **`llama.cpp` upstream `mtmd_*` migration:** upstream llama.cpp deprecated `llava-cli` / `mmproj-cli` in favour of `mtmd_*` (multimodal tensor decode) starting mid-2025. Whether the `llama-cpp-2` 0.1.x **Rust bindings** re-exported `mtmd_*` symbols, or stayed on the older `clip_*` family until a 0.2.x bump, is the **single most important unknown** for this plan (§5).
- **`candle-transformers` rejected** despite being a clean pure-Rust path: it doesn't read `mmproj-*.gguf` at all, so we'd have to ship a **second** model artifact (safetensors LLaVA tower) just to do vision, defeating the catalog's GGUF-only contract.
- **`mistral.rs` rejected** for similar reasons plus dedupe risk: pulls its own `cudarc` / `metal-rs` toolchain that may conflict with `llama-cpp-2`'s `llama-cpp-sys-2` build.

---

## 3. Best-fit recommendation

**Stay on `llama-cpp-2` 0.1.x. Wrap mmproj inside `services/llama.rs`.**

Justification, keyed to the architecture rules:

1. **In-process only (rule 1):** llama-cpp-2 already runs in our process; mmproj loads as a second context via the same FFI. No subprocess needed.
2. **Pinned crate dedupe (rule 2):** zero new transitive deps. The plugin's transitive copy stays the only sys-crate, so `llama-cpp-sys-2` still links once. **If 0.1.x does not expose the mmproj/mtmd entry points we need (see §5 unknown #1), we must escalate the version bump as a blocker rather than swap crates** — every alternative cascades worse than a minor bump.
3. **No new plugin (rule 3):** lives in `services/llama.rs` next to `load_model` / `start_session`. New `services/mmproj_download.rs` is a plain module, not a plugin.
4. **Caller-aware audit (rule 4):** `viboinference_attach_image` Tauri command threads `CallerContext` exactly like the existing `download_model` / `load_model` commands do.
5. **No new HTTP plugin (rule 5):** `services/mmproj_download.rs` reuses `reqwest = "0.12"`.
6. **Stronghold-friendly (rule 6):** image bytes never leave Rust as keys; mmproj weights are public files, not secrets.
7. **Mobile out of scope (rule 7):** new code lives behind the same `cfg(not(any(target_os = "android", target_os = "ios")))` block that already gates `tauri-plugin-leap-ai` and `llama-cpp-2`.

Word budget: ~190.

---

## 4. Minimal-touch integration plan

### 4.1 File map

| Status   | Path                                                | Purpose                                                                  |
| -------- | --------------------------------------------------- | ------------------------------------------------------------------------ |
| **NEW**  | `src-tauri/src/services/mmproj_download.rs`         | Resolve catalog `mmproj_url` → on-disk path; uses `reqwest`              |
| **EDIT** | `src-tauri/src/services/llama.rs`                   | Add `MmprojCtx`, `load_mmproj`, `encode_image`; thread mmproj through `start_session` |
| **EDIT** | `src-tauri/src/services/mod.rs`                     | Re-export `mmproj_download`                                              |
| **EDIT** | `src-tauri/src/services/model_catalog.rs`           | Fill `mmproj_url: Some(...)` on `lfm2.5-vl-450m`; flip `enabled: true`   |
| **EDIT** | Tauri command surface (`commands.rs` or equivalent) | Register `viboinference_attach_image`                                    |
| **EDIT** | `src/services/inferenceClient.ts` (FE shim)         | Thin wrapper around the new command (out of Rust scope but called out)  |

### 4.2 Catalog patch (smallest possible diff)

```rust
// src-tauri/src/services/model_catalog.rs — VL entry, after research:
limits: ModelLimits {
    n_ctx_max: 32_768,
    default_n_ctx: 16_384,
    supports_vision: true,
    supports_thinking: false,
    // F16 mmproj is the canonical companion shard for the Q4_0 LM on
    // LiquidAI/LFM2.5-VL-450M-GGUF. Final filename pending §5 unknown #3
    // verification — pattern follows HF mmproj convention.
    mmproj_url: Some(
        "https://huggingface.co/LiquidAI/LFM2.5-VL-450M-GGUF/resolve/main/mmproj-LFM2.5-VL-450M-F16.gguf"
    ),
},
// + flip `enabled: false` -> true once this whole plan lands.
```

The forward-reference test `vision_model_is_disabled_until_phase_f2` flips
to `assert!(vl.enabled)` in the same commit that flips the field, satisfying
the canary contract.

### 4.3 `services/mmproj_download.rs` (new, ~80 LoC)

Concrete signatures:

```rust
//! Companion-shard downloader for vision models. Mirrors the leap-ai
//! plugin's GGUF cache layout but for `mmproj-*.gguf`. Reuses reqwest;
//! no new HTTP dep.

use std::path::{Path, PathBuf};
use crate::models::CallerContext;

#[derive(Debug, thiserror::Error)]
pub enum MmprojError {
    #[error("network: {0}")] Net(#[from] reqwest::Error),
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("checksum mismatch (got {got}, want {want})")]
    Checksum { got: String, want: String },
}

/// Idempotent: returns the cached path if already on disk.
/// `caller` is logged for audit (User vs Agent) — never gates download.
pub async fn ensure_cached(
    cache_dir: &Path,
    url: &str,
    caller: &CallerContext,
) -> Result<PathBuf, MmprojError> { /* ~30 lines */ }

fn cache_filename_for(url: &str) -> String { /* hash url -> stable filename */ }
```

Sketch of `ensure_cached` body:

```rust
let dst = cache_dir.join(cache_filename_for(url));
if dst.exists() { log::info!("[mmproj] cache hit caller={caller:?} path={dst:?}"); return Ok(dst); }
log::info!("[mmproj] downloading caller={caller:?} url={url}");
let bytes = reqwest::Client::new().get(url).send().await?.error_for_status()?.bytes().await?;
let tmp = dst.with_extension("partial");
tokio::fs::write(&tmp, &bytes).await?;
tokio::fs::rename(&tmp, &dst).await?;
Ok(dst)
```

### 4.4 `services/llama.rs` additions (~60 LoC delta)

```rust
/// Loaded mmproj projector. Held next to `LoadedModel`; lifetime tied
/// to the LM it pairs with (clip/mtmd ctx pins resources we cannot
/// move across LM swaps).
pub struct MmprojCtx {
    // EXACT FIELD DEPENDS ON §5 UNKNOWN #1:
    //   variant A (mtmd available in 0.1.x): mtmd_ctx: llama_cpp_2::mtmd::MtmdContext,
    //   variant B (only legacy clip in 0.1.x): clip_ctx: llama_cpp_2::clip::ClipContext,
    //   variant C (neither — blocker, see §5/§7): triggers 0.2.x bump escalation.
}

pub struct ImageTokens {
    /// Embedding rows ready to splice into a LlamaBatch as image-token slots.
    pub embeddings: Vec<f32>,
    /// Number of image tokens this image expanded into (model-dependent;
    /// for SigLIP-class projectors typically 256–576).
    pub n_tokens: usize,
}

impl LlamaService {
    /// Load the mmproj shard. Mirrors `load_model`'s style — not async;
    /// blocks the caller for the duration of the ggml load. Stash next to
    /// the LoadedModel so a session can fetch both with one lock.
    pub fn load_mmproj(&self, our_id: &str, path: &Path) -> Result<(), String> {
        let backend = Self::backend();
        // Variant A:
        //   let m = MtmdContext::load_from_file(&backend, path)
        //       .map_err(|e| format!("mtmd load: {e}"))?;
        // Variant B:
        //   let m = ClipContext::load_from_file(path)
        //       .map_err(|e| format!("clip load: {e}"))?;
        // self.mmproj.lock().unwrap().insert(our_id.into(), Arc::new(MmprojCtx { .. }));
        Ok(())
    }

    /// Run the projector over PNG bytes and return token slots.
    pub fn encode_image(
        &self,
        our_id: &str,
        png_bytes: &[u8],
    ) -> Result<ImageTokens, String> {
        // 1. Decode PNG -> RGB8 with `image` crate (light; ~150 KB) OR rely
        //    on llama-cpp-2's clip helper that takes raw bytes.
        // 2. Call mtmd_encode / clip_image_encode -> Vec<f32> embeddings.
        // 3. Wrap in ImageTokens.
        Ok(ImageTokens { embeddings: vec![], n_tokens: 0 })
    }

    /// Modify start_session to accept an optional pre-encoded image. The
    /// image tokens get prepended to the user-turn batch (after the chat
    /// template renders an `<image>` placeholder that the renderer leaves
    /// in place — LFM2.5-VL's template uses `<image>` literally).
    pub fn start_session_with_image(
        &self, our_id: &str, system: Option<String>,
        cfg: SessionConfig, image: Option<ImageTokens>,
    ) -> Result<String, String> {
        // For the minimum viable path, image stays attached to the session
        // and is consumed by the next stream() call. ~10 lines diff vs
        // existing start_session.
        unimplemented!()
    }
}
```

The decode loop in `run_decode` needs one branch: when the session has a
pending `ImageTokens`, splice `n_tokens` image-embedding slots into the
prompt batch *before* the text tokens, using the existing `LlamaBatch`
API. That's ~15 LoC and the only structural change to a working code path.

### 4.5 New Tauri command

```rust
// In commands.rs (or wherever inference commands are registered):
#[tauri::command]
pub async fn viboinference_attach_image<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, Arc<LlamaService>>,
    session_id: String,
    png_bytes: Vec<u8>,            // serde_bytes for compactness
    caller: CallerContext,
) -> Result<AttachImageResp, String> {
    // 1. Look up the session's model id.
    // 2. Ensure mmproj for that model is loaded; if not, return a typed
    //    error the FE can map to "download mmproj first".
    // 3. service.encode_image(&model_id, &png_bytes)
    // 4. Stash on the session so the next stream() consumes it.
    // 5. Return { n_tokens, ctx_used_pct } so the UI can render a chip.
}
```

FE shim adds two lines in `inferenceClient.ts` (out of Rust scope).

### 4.6 What forces a major version bump?

- If `llama-cpp-2` 0.1.x re-exports neither `mtmd_*` nor `clip_*` that we
  can call (§5 unknown #1 resolves "no"), then **we must bump to whatever
  version first wraps the multimodal C API**. That bump cascades to:
  - `tauri-plugin-leap-ai = "0.1.1"` — its transitive `llama-cpp-2` pin
    will dedupe only if the major matches. **A 0.2.x bump on our side
    while the plugin stays on 0.1.x compiles two copies of `llama-cpp-sys-2`
    — link failure.** We'd have to either (a) wait for the plugin to bump,
    (b) fork the plugin, or (c) drop the plugin and inline its
    download-only path into our own service. Option (c) is the cleanest
    escape hatch and roughly matches the work already done in T11
    (removing dead `ModelManager`).

This is the primary blocker scenario; §7 risk #1 covers mitigation.

---

## 5. Open questions / unknowns

1. **Does `llama-cpp-2` 0.1.x expose `mtmd_*` or `clip_*` Rust APIs?**
   Couldn't reach crates.io / docs.rs / the utilityai GitHub repo during
   this pass (WebFetch + WebSearch denied). Resolution path: clone or
   `cargo doc` the locally-cached source under `~/.cargo/registry/src/`
   and grep for `mtmd_init`, `clip_image_load`, `clip_image_encode`,
   `llama_clip_*`. **Block Phase F2 plan-approval until this is
   confirmed.**

2. **Is there a 0.2.x `llama-cpp-2` line, and what does it cascade to?**
   Same reachability gap as #1. Resolution: `cargo search llama-cpp-2`
   plus visual inspection of `tauri-plugin-leap-ai` 0.1.1's `Cargo.toml`
   to see its `llama-cpp-2 = ">=…"` requirement.

3. **HuggingFace `LiquidAI/LFM2.5-VL-450M-GGUF` mmproj filename.** The
   plan above guesses `mmproj-LFM2.5-VL-450M-F16.gguf` based on the
   community convention (`mmproj-*.gguf`, F16 default). **Verify before
   landing the catalog patch** — a 404 here is a UX disaster on first
   download.

4. **LFM 2.5-VL chat-template image placeholder.** Many VL models embed
   a literal `<image>` token; some use `<|image|>` or `<|vision_start|>`.
   We render the chat template via minijinja today — if the template
   strips/escapes the placeholder we need to short-circuit it for VL.
   Verify by `LlamaModel::chat_template` dump on the downloaded GGUF.

5. **Image preprocessing dependency.** If we decode PNG ourselves before
   handing bytes to the projector, we add `image = "0.25"` (~150 KB
   compiled). If `llama-cpp-2`'s clip helper takes raw PNG bytes, we
   skip it. Tied to unknown #1.

6. **Memory ceiling for VL on the M-series default tier.** 450M LM +
   mmproj F16 (~370 MB) + 16 k context KV is comfortable; if a future VL
   model bumps mmproj to BF16/F32 we'll need a sentinel.

---

## 6. Effort estimate

| Sub-task                                                | LoC    | Days  | Notes                                                                |
| ------------------------------------------------------- | ------ | ----- | -------------------------------------------------------------------- |
| **(a) `services/mmproj_download.rs`**                   | ~80    | 0.5   | Mirrors existing `reqwest` patterns; trivial cache.                 |
| **(b) `services/llama.rs` mmproj wrapper**              | ~150   | 1.5   | Bulk of risk lives here — exact API depends on §5 unknowns.         |
| **(c) Tauri command + FE shim**                         | ~80    | 0.5   | One command, one TS function, one event passthrough.                |
| **(d) UI image-attach affordance**                      | ~120   | 1.0   | Paperclip button → file picker → base64 → command call → chip.      |
| **catalog patch + flip canary test**                    | ~10    | 0.0   | Lands in the same PR.                                               |
| **manual QA on M-series + Intel Mac**                   | n/a    | 0.5   | "Describe this image", token-rate sanity, OOM probe.                |
| **Total (no blocker)**                                  | ~440   | **4.0** | Realistic; gated entirely on §5 unknown #1.                       |
| **Total (if blocker — see §7 risk #1)**                 | ~440 + plugin extraction (~300) | **+3.0** | Assumes we extract leap-ai's download path inline. |

---

## 7. Risk register

| #   | Risk                                                                                                                                                                                                                                               | Likelihood | Impact   | Mitigation                                                                                                                                                                                                                              |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | **0.1.x doesn't expose multimodal API, and bumping forces a fork of `tauri-plugin-leap-ai`.** This is the single highest-impact risk: the plugin's transitive `llama-cpp-2` pin would cause double-linking after a 0.2.x bump on our side.         | Medium     | High     | Spike before phase plan: confirm §5 unknown #1 by inspecting the cached crate source. If negative, scope a "drop leap-ai plugin, inline its download cache in `services/leap_download.rs`" sub-phase before F2 — already partly done in T11. |
| 2   | **mmproj filename on HF doesn't match the guessed pattern**, so first-run download 404s and the user sees a generic error.                                                                                                                         | Medium     | Medium   | Verify by HEAD-request before committing the catalog `mmproj_url`; gate behind a `cargo test` that hits HF (network-flagged).                                                                                                          |
| 3   | **Chat template strips/escapes the image placeholder under minijinja**, so prompt-time image splicing inserts tokens at the wrong position and the model hallucinates.                                                                             | Low–Medium | Medium   | Add a regression test that renders the LFM2.5-VL template with a known message vector and asserts the placeholder survives. If it doesn't, branch to a manual splice path that bypasses minijinja for VL only.                          |

---

## Appendix A — citations and reachability log

- **Could not reach** crates.io, docs.rs, github.com/utilityai/llama-cpp-rs, or huggingface.co/LiquidAI/LFM2.5-VL-450M-GGUF during this pass: WebFetch and WebSearch were denied. All version / API claims above are flagged with confidence and the §5 unknowns enumerate exactly what to verify before promotion to a phase plan.
- **In-repo evidence consulted:**
  - `src-tauri/Cargo.toml` lines 60–66 (pins `llama-cpp-2 = "0.1"`, plugin at `0.1.1`).
  - `src-tauri/src/services/llama.rs` (full file) — gives the exact extension surface for `MmprojCtx` / `encode_image` / `start_session_with_image`.
  - `src-tauri/src/services/model_catalog.rs` lines 215–242 + tests 314–323 — VL entry shape, canary test contract.
  - `src-tauri/src/models/mod.rs` lines 5–10 — `CallerContext` shape used by `mmproj_download`.
- **Forward references not yet updated** (deliberately left to Phase F2 commit):
  - `model_catalog.rs` doc comment on lines 19–23 cites `VISION_RESEARCH_2026-04-28.md`. Today's report is `VISION_RESEARCH_2026-04-30.md`. Defer the rename.
