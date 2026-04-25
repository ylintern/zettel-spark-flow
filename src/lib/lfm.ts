/**
 * LFM Unified Chat Client
 *
 * Routes to local on-device LFM (via Rust `InferenceService` → `tauri-plugin-leap-ai`)
 * or to a configured cloud provider, based on the active provider stored in
 * `models.ts` (`getActiveProvider()`).
 *
 * The local branch goes through `@/lib/inference` — there is NO HTTP fetch
 * to a localhost LEAP server anymore (the plugin runs llama.cpp in-process).
 * Cloud branch is unchanged.
 *
 * Public API kept stable for callers (`ChatAssistant.tsx`):
 *   - `streamLfmChat({ messages, onDelta, onDone, onError, signal, localSessionRef })`
 *   - `isLfmConfigured(): boolean`            — sync, cached
 *   - `refreshLocalReady(): Promise<void>`    — refresh the local cache
 *   - `getActiveProviderLabel(): string`      — sync, cached
 *
 * Local-readiness is async (we have to ask Rust whether a model is active),
 * so it is exposed as a cached sync getter plus an async refresher. Callers
 * that already poll on an interval (e.g. `ChatAssistant`'s 2s loop) should
 * call `refreshLocalReady()` from inside that loop.
 */

import { onCloudStreamEvent, streamCloudMessage } from "@/lib/commands";
import {
  getActiveProvider,
  getCloudKeys,
  loadCloudKeys,
  CLOUD_PROVIDERS,
  getSelectedModels,
} from "@/lib/models";
import {
  endChatSession,
  getActiveModel,
  listModels,
  startChatSession,
  streamChat,
  type ModelEntryDto,
} from "@/lib/inference";

export interface LfmMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

const SYSTEM_PROMPT: LfmMessage = {
  role: "system",
  content: `You are ViBo Assistant, a private AI running locally via LFM. You help users organize notes, create tasks, and think through ideas. You are concise, helpful, and respect user privacy — all data stays on-device. If the user says "new note: <title>" or "new task: <title>", acknowledge it and confirm creation.`,
};

// ── Local-readiness cache ───────────────────────────────────────────────
//
// `isLfmConfigured()` and `getActiveProviderLabel()` are called in render,
// so they must be synchronous. We back them with a cache that is refreshed
// out-of-band by `refreshLocalReady()`.

let _localActiveId: string | null = null;
let _localAlias: string | null = null;
let _modelCatalog: ModelEntryDto[] | null = null;

async function ensureCatalog(): Promise<ModelEntryDto[]> {
  if (_modelCatalog) return _modelCatalog;
  try {
    _modelCatalog = await listModels();
  } catch (e) {
    console.error("[lfm] listModels failed:", e);
    _modelCatalog = [];
  }
  return _modelCatalog;
}

/** Refresh the cached local-readiness flag. Safe to call repeatedly. */
export async function refreshLocalReady(): Promise<void> {
  try {
    const [activeId, catalog] = await Promise.all([
      getActiveModel(),
      ensureCatalog(),
    ]);
    _localActiveId = activeId;
    _localAlias = activeId
      ? catalog.find((m) => m.id === activeId)?.alias ?? null
      : null;
  } catch (e) {
    console.error("[lfm] refreshLocalReady failed:", e);
    _localActiveId = null;
    _localAlias = null;
  }
}

export function isLfmConfigured(): boolean {
  const provider = getActiveProvider();
  if (provider === "local") return _localActiveId != null;
  const keys = getCloudKeys();
  return !!keys[provider];
}

export function getActiveProviderLabel(): string {
  const provider = getActiveProvider();
  if (provider === "local") {
    return _localAlias ? `Local · ${_localAlias}` : "Local LFM";
  }
  const found = CLOUD_PROVIDERS.find((p) => p.id === provider);
  return found?.name || provider;
}

// ── Streaming entry point ───────────────────────────────────────────────

/**
 * Stream a chat completion from the configured provider.
 *
 * For the local path, the caller must supply a `localSessionRef`. The session
 * is created on first use (with the system prompt) and reused for subsequent
 * sends — the plugin tracks history inside the conversation. Callers MUST
 * call `endChatSession(ref.current)` and reset `ref.current = null` on:
 *   - component unmount,
 *   - chat-session prop change,
 *   - active-model change (so the next send rebinds to the new model).
 */
export async function streamLfmChat({
  messages,
  onDelta,
  onDone,
  onError,
  signal,
  localSessionRef,
}: {
  messages: LfmMessage[];
  onDelta: (text: string) => void;
  onDone: () => void;
  onError: (err: string) => void;
  signal?: AbortSignal;
  localSessionRef?: { current: string | null };
}) {
  const provider = getActiveProvider();
  await loadCloudKeys().catch(() => undefined);

  if (provider === "local") {
    // Make sure the local cache is fresh before we decide.
    await refreshLocalReady();
    if (!_localActiveId) {
      onError("No local model active. Open Settings → Local Models to download or activate one.");
      return;
    }
    const ref = localSessionRef ?? { current: null as string | null };
    try {
      if (!ref.current) {
        ref.current = await startChatSession(SYSTEM_PROMPT.content);
      }
      // Plugin manages multi-turn history inside the conversation, so we
      // only send the latest user turn as the prompt.
      const lastUser = [...messages].reverse().find((m) => m.role === "user");
      const prompt = lastUser?.content ?? "";
      await streamChat({
        sessionId: ref.current,
        prompt,
        onDelta,
        signal,
      });
      onDone();
    } catch (err: any) {
      if (err?.name === "AbortError") {
        onDone();
        return;
      }
      onError(err?.message ?? String(err));
    }
    return;
  }

  // ── Cloud path (unchanged) ────────────────────────────────────────────
  try {
    const selectedModel = getSelectedModels()[provider] || null;
    const requestId = await streamCloudMessage(provider, selectedModel, [
      SYSTEM_PROMPT,
      ...messages,
    ]);
    let unlistenFn: (() => Promise<void>) | null = null;
    unlistenFn = await onCloudStreamEvent((event) => {
      if (event.requestId !== requestId) return;
      if (event.delta) onDelta(event.delta);
      if (event.error) {
        void unlistenFn?.();
        onError(event.error);
      }
      if (event.done) {
        void unlistenFn?.();
        onDone();
      }
    });
    signal?.addEventListener("abort", () => {
      void unlistenFn?.();
      onDone();
    });
  } catch (err: any) {
    if (err?.name === "AbortError") {
      onDone();
      return;
    }
    onError(`Cannot reach ${getActiveProviderLabel()}. Is it running?`);
  }
}
