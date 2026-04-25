/**
 * Vibo Inference Client
 *
 * Thin typed wrapper around the `viboinference_*` Tauri commands exposed by
 * `src-tauri/src/commands/inference.rs`. Every call goes through our Rust
 * service (`InferenceService`), which is the single choke point for local
 * LLM inference. The plugin's raw `leap-ai://event` stream is translated to
 * `vibo://*` events on the Rust side so this file never sees plugin-native
 * event shapes.
 *
 * This replaces `src/lib/lfm.ts` (the old HTTP-to-external-LEAP-server
 * client). Cloud-provider streaming still lives elsewhere
 * (`streamCloudMessage` in `src/lib/commands.ts`).
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ModelEntryDto {
  id: string;
  name: string;
  /** Short role-name used by the agent layer ("junior", "specialist"). */
  alias: string;
  family: string;
  params: string;
  sizeMb: number;
  modality: string[];
  description: string;
  recommended: boolean;
  quantization: string;
}

// ── Event payload types (match Rust serde_rename_all = "camelCase") ──

export interface ModelDownloadProgressEvent {
  modelId: string;
  progress: number; // 0.0 → 1.0
  status?: string;
}

export interface ModelStateEvent {
  modelId: string;
  state: "model-loaded" | "model-unloaded" | "download-removed" | string;
}

export interface ChatDeltaEvent {
  sessionId: string;
  generationId: string;
  delta: string;
}

export interface ChatDoneEvent {
  sessionId: string;
  generationId: string;
  error?: string;
}

export interface SessionInfo {
  modelAlias: string;
  nCtx: number;
  usedTokens: number;
  maxTokens: number;
  temperature: number;
}

// ── Commands ─────────────────────────────────────────────────────────

export function listModels(): Promise<ModelEntryDto[]> {
  return invoke<ModelEntryDto[]>("viboinference_list_models");
}

export function listDownloaded(): Promise<string[]> {
  return invoke<string[]>("viboinference_list_downloaded");
}

export function downloadModel(id: string): Promise<void> {
  return invoke<void>("viboinference_download_model", { id });
}

export function deleteModel(id: string): Promise<void> {
  return invoke<void>("viboinference_delete_model", { id });
}

export function getActiveModel(): Promise<string | null> {
  return invoke<string | null>("viboinference_get_active_model");
}

export function setActiveModel(id: string): Promise<void> {
  return invoke<void>("viboinference_set_active_model", { id });
}

export function startChatSession(systemPrompt?: string): Promise<string> {
  return invoke<string>("viboinference_start_chat_session", {
    systemPrompt: systemPrompt ?? null,
  });
}

export function streamChatRaw(
  sessionId: string,
  prompt: string,
): Promise<string> {
  return invoke<string>("viboinference_stream_chat", { sessionId, prompt });
}

export function stopGeneration(generationId: string): Promise<void> {
  return invoke<void>("viboinference_stop_generation", { generationId });
}

export function endChatSession(sessionId: string): Promise<void> {
  return invoke<void>("viboinference_end_chat_session", { sessionId });
}

export function getSessionInfo(sessionId: string): Promise<SessionInfo | null> {
  return invoke<SessionInfo | null>("viboinference_session_info", { sessionId });
}

// ── Event subscribers ────────────────────────────────────────────────

export async function onModelDownloadProgress(
  handler: (ev: ModelDownloadProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<ModelDownloadProgressEvent>(
    "vibo://model-download-progress",
    (e) => handler(e.payload),
  );
}

export async function onModelState(
  handler: (ev: ModelStateEvent) => void,
): Promise<UnlistenFn> {
  return listen<ModelStateEvent>("vibo://model-state", (e) =>
    handler(e.payload),
  );
}

export async function onChatDelta(
  handler: (ev: ChatDeltaEvent) => void,
): Promise<UnlistenFn> {
  return listen<ChatDeltaEvent>("vibo://chat-delta", (e) => handler(e.payload));
}

export async function onChatDone(
  handler: (ev: ChatDoneEvent) => void,
): Promise<UnlistenFn> {
  return listen<ChatDoneEvent>("vibo://chat-done", (e) => handler(e.payload));
}

// ── High-level helpers ───────────────────────────────────────────────

/**
 * Stream a single prompt through an existing chat session. Resolves when
 * `vibo://chat-done` fires for this generation (or when `signal` aborts).
 *
 * The caller owns the session lifecycle — use `startChatSession` /
 * `endChatSession` to create and tear down.
 */
export async function streamChat({
  sessionId,
  prompt,
  onDelta,
  signal,
}: {
  sessionId: string;
  prompt: string;
  onDelta: (delta: string) => void;
  signal?: AbortSignal;
}): Promise<void> {
  let unlistenDelta: UnlistenFn | null = null;
  let unlistenDone: UnlistenFn | null = null;
  let generationId: string | null = null;

  const cleanup = () => {
    void unlistenDelta?.();
    void unlistenDone?.();
  };

  return new Promise<void>((resolve, reject) => {
    Promise.all([
      onChatDelta((ev) => {
        if (ev.sessionId === sessionId && ev.generationId === generationId) {
          onDelta(ev.delta);
        }
      }),
      onChatDone((ev) => {
        if (ev.sessionId === sessionId && ev.generationId === generationId) {
          cleanup();
          if (ev.error) reject(new Error(ev.error));
          else resolve();
        }
      }),
    ])
      .then(([dUn, doneUn]) => {
        unlistenDelta = dUn;
        unlistenDone = doneUn;
        return streamChatRaw(sessionId, prompt);
      })
      .then((gid) => {
        generationId = gid;
        if (signal) {
          signal.addEventListener("abort", () => {
            if (generationId) void stopGeneration(generationId);
            cleanup();
            resolve();
          });
        }
      })
      .catch((err) => {
        cleanup();
        reject(err);
      });
  });
}
