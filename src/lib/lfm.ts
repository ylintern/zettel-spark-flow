/**
 * LFM Unified Chat Client
 * Routes to local LFM (LEAP) or cloud providers based on active config.
 */

import { onCloudStreamEvent, streamCloudMessage } from "@/lib/commands";
import {
  getActiveProvider,
  getActiveLocalModel,
  getLfmEndpoint,
  getCloudKeys,
  loadCloudKeys,
  CLOUD_PROVIDERS,
  getDownloadedModels,
  getSelectedModels,
  type ActiveProvider,
} from "@/lib/models";

export interface LfmMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

const SYSTEM_PROMPT: LfmMessage = {
  role: "system",
  content: `You are ViBo Assistant, a private AI running locally via LFM. You help users organize notes, create tasks, and think through ideas. You are concise, helpful, and respect user privacy — all data stays on-device. If the user says "new note: <title>" or "new task: <title>", acknowledge it and confirm creation.`,
};

export function isLfmConfigured(): boolean {
  const provider = getActiveProvider();
  if (provider === "local") {
    return !!getLfmEndpoint() && getDownloadedModels().length > 0;
  }
  const keys = getCloudKeys();
  return !!keys[provider];
}

export function getActiveProviderLabel(): string {
  const provider = getActiveProvider();
  if (provider === "local") return "LFM Local";
  const found = CLOUD_PROVIDERS.find(p => p.id === provider);
  return found?.name || provider;
}

function getLocalEndpointAndModel(): { url: string; model?: string } {
  const endpoint = getLfmEndpoint();
  return {
    url: `${endpoint.replace(/\/+$/, "")}/v1/chat/completions`,
    model: getActiveLocalModel(),
  };
}

/**
 * Stream a chat completion from the configured provider.
 */
export async function streamLfmChat({
  messages,
  onDelta,
  onDone,
  onError,
  signal,
}: {
  messages: LfmMessage[];
  onDelta: (text: string) => void;
  onDone: () => void;
  onError: (err: string) => void;
  signal?: AbortSignal;
}) {
  const provider = getActiveProvider();
  await loadCloudKeys().catch(() => undefined);

  if (provider === "local" && !getLfmEndpoint()) {
    onError("LEAP server not configured. Go to Settings → Local Models to set the endpoint.");
    return;
  }

  try {
    if (provider === "local") {
      const { url, model } = getLocalEndpointAndModel();
      const body: any = {
        messages: [SYSTEM_PROMPT, ...messages],
        stream: true,
        max_tokens: 1024,
      };
      if (model) body.model = model;
      const resp = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
        signal,
      });
      if (!resp.ok) {
        const text = await resp.text().catch(() => "");
        onError(`${getActiveProviderLabel()} error (${resp.status}): ${text || "Connection failed"}`);
        return;
      }
      if (!resp.body) {
        onError("No response body");
        return;
      }
      const reader = resp.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        let newlineIdx: number;
        while ((newlineIdx = buffer.indexOf("\n")) !== -1) {
          let line = buffer.slice(0, newlineIdx);
          buffer = buffer.slice(newlineIdx + 1);
          if (line.endsWith("\r")) line = line.slice(0, -1);
          if (line.startsWith(":") || line.trim() === "" || !line.startsWith("data: ")) continue;
          const jsonStr = line.slice(6).trim();
          if (jsonStr === "[DONE]") break;
          try {
            const parsed = JSON.parse(jsonStr);
            const content = parsed.choices?.[0]?.delta?.content || "";
            if (content) onDelta(content);
          } catch {
            buffer = line + "\n" + buffer;
            break;
          }
        }
      }
      onDone();
      return;
    }

    const selectedModel = getSelectedModels()[provider] || null;
    const requestId = await streamCloudMessage(provider, selectedModel, [SYSTEM_PROMPT, ...messages]);
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
    if (err.name === "AbortError") {
      onDone();
      return;
    }
    onError(`Cannot reach ${getActiveProviderLabel()}. Is it running?`);
  }
}
