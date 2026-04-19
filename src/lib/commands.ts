import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Note, KanbanColumn } from "./types";

export interface WorkspaceSnapshot {
  notes: Note[];
  folders: string[];
  columns: KanbanColumn[];
}

interface SecretValue {
  value: string | null;
}

export interface NoteIndexingProgressEvent {
  noteId: string | null;
  stage: string;
  progress: number;
  processedNotes: number;
  totalNotes: number;
}

export interface AgentThinkingDeltaEvent {
  sessionId: string | null;
  delta: string;
  done: boolean;
}

export interface VaultStatusChangedEvent {
  configured: boolean;
  unlocked: boolean;
  reason: string;
}

export interface CloudMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface CloudStreamEvent {
  requestId: string;
  delta?: string;
  done: boolean;
  error?: string;
}

export interface DeviceCapabilities {
  platform: string;
  isDesktop: boolean;
  isMobile: boolean;
  hasSecureEnclave: boolean;
  hasTouchId: boolean;
  hasFaceId: boolean;
  supportsBiometricPrompt: boolean;
  canOfferBiometrics: boolean;
  supportsPin: boolean;
  supportsPassphrase: boolean;
}

/**
 * Runtime feature flags returned by `config::features::get_feature_flags`.
 *
 * Phase 0 (current): all flags false — encryption/biometric/iOS code paths
 * remain compiled in but are bypassed. Flipping a flag in Rust and recompiling
 * re-activates the original behavior end-to-end, no code resurrection needed.
 *
 * Field names intentionally match the Rust struct (serde default serialization,
 * no rename). See src-tauri/src/config/features.rs.
 */
export interface FeatureFlags {
  encryption_enabled: boolean;
  biometric_enabled: boolean;
  ios_enabled: boolean;
}

/** Safe Phase 0 defaults for contexts without a Tauri runtime (web preview). */
export const PHASE_0_FEATURE_FLAGS: FeatureFlags = {
  encryption_enabled: false,
  biometric_enabled: false,
  ios_enabled: false,
};

export function isTauriRuntimeAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

export async function loadWorkspaceSnapshot(): Promise<WorkspaceSnapshot> {
  return tauriInvoke<WorkspaceSnapshot>("load_workspace_snapshot");
}

export async function saveWorkspaceNote(note: Note): Promise<void> {
  return tauriInvoke<void>("save_note", { note, caller: { type: "user" } });
}

export async function deleteWorkspaceNote(id: string): Promise<void> {
  return tauriInvoke<void>("delete_note", { id, caller: { type: "user" } });
}

export async function createWorkspaceFolder(name: string): Promise<void> {
  return tauriInvoke<void>("create_folder", { name, caller: { type: "user" } });
}

export async function saveWorkspaceColumn(column: KanbanColumn): Promise<void> {
  return tauriInvoke<void>("save_column", { column, caller: { type: "user" } });
}

export async function deleteWorkspaceColumn(id: string): Promise<void> {
  return tauriInvoke<void>("delete_column", { id, caller: { type: "user" } });
}

export async function setupSecureVault(passphrase: string): Promise<void> {
  return tauriInvoke<void>("setup_secure_vault", { passphrase });
}

export async function unlockSecureVault(passphrase: string): Promise<void> {
  return tauriInvoke<void>("unlock_vault", { passphrase });
}

export async function lockSecureVault(): Promise<void> {
  return tauriInvoke<void>("lock_vault");
}

export async function isVaultUnlocked(): Promise<boolean> {
  return tauriInvoke<boolean>("is_vault_unlocked");
}

export async function isVaultConfigured(): Promise<boolean> {
  return tauriInvoke<boolean>("is_vault_configured");
}

export async function storeSecretInVault(key: string, value: string): Promise<void> {
  return tauriInvoke<void>("store_secret", { key, value });
}

export async function getSecretFromVault(key: string): Promise<string | null> {
  const result = await tauriInvoke<SecretValue>("get_secret", { key });
  return result.value;
}

export async function deleteSecretFromVault(key: string): Promise<void> {
  return tauriInvoke<void>("delete_secret", { key });
}

export async function getProviderStatus(provider: string): Promise<boolean> {
  return tauriInvoke<boolean>("get_provider_status", { provider });
}

export async function streamCloudMessage(
  provider: string,
  model: string | null,
  messages: CloudMessage[],
): Promise<string> {
  return tauriInvoke<string>("stream_cloud_message", { provider, model, messages });
}

export async function exportNotes(): Promise<string> {
  return tauriInvoke<string>("export_notes", { caller: { type: "user" } });
}

export async function factoryReset(): Promise<void> {
  return tauriInvoke<void>("factory_reset");
}

export async function getDeviceCapabilities(): Promise<DeviceCapabilities> {
  return tauriInvoke<DeviceCapabilities>("get_device_capabilities");
}

export async function verifyBiometricUnlock(): Promise<boolean> {
  return tauriInvoke<boolean>("verify_biometric_and_unlock");
}

export async function fallbackPassphraseUnlock(passphrase: string): Promise<void> {
  return tauriInvoke<void>("fallback_passphrase_unlock", { passphrase });
}

/**
 * Fetch runtime feature flags from the Rust backend.
 *
 * Frontend uses this on mount to decide whether to mount LockScreen, render
 * lock icons, show biometric Settings, etc. All rendering that depends on
 * encryption/biometric/iOS behavior should be gated by the returned flags.
 */
export async function getFeatureFlags(): Promise<FeatureFlags> {
  return tauriInvoke<FeatureFlags>("get_feature_flags");
}

async function subscribeToEvent<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntimeAvailable()) {
    return async () => {};
  }

  return listen<T>(event, (payload) => handler(payload.payload));
}

export async function onNoteIndexingProgress(
  handler: (payload: NoteIndexingProgressEvent) => void,
): Promise<UnlistenFn> {
  return subscribeToEvent<NoteIndexingProgressEvent>("note_indexing_progress", handler);
}

export async function onAgentThinkingDelta(
  handler: (payload: AgentThinkingDeltaEvent) => void,
): Promise<UnlistenFn> {
  return subscribeToEvent<AgentThinkingDeltaEvent>("agent_thinking_delta", handler);
}

export async function onVaultStatusChanged(
  handler: (payload: VaultStatusChangedEvent) => void,
): Promise<UnlistenFn> {
  return subscribeToEvent<VaultStatusChangedEvent>("vault_status_changed", handler);
}

export async function onCloudStreamEvent(
  handler: (payload: CloudStreamEvent) => void,
): Promise<UnlistenFn> {
  return subscribeToEvent<CloudStreamEvent>("cloud_message_stream", handler);
}
