import {
  isVaultConfigured,
  isVaultUnlocked,
  lockSecureVault,
  setupSecureVault,
  unlockSecureVault,
} from "./commands";

export async function setupPin(pin: string): Promise<void> {
  await setupSecureVault(pin);
}

export async function verifyPin(pin: string): Promise<boolean> {
  try {
    await unlockSecureVault(pin);
    return true;
  } catch {
    return false;
  }
}

export async function isPinSetup(): Promise<boolean> {
  return isVaultConfigured();
}

export async function isVaultReady(): Promise<boolean> {
  return isVaultUnlocked();
}

export async function lockVault(): Promise<void> {
  await lockSecureVault();
}

// Agent notes are stored separately, unencrypted (agents always have access)
const AGENT_NOTES_KEY = "vibo-agent-notes";

export function loadAgentNotes(): string {
  return localStorage.getItem(AGENT_NOTES_KEY) || "[]";
}

export function saveAgentNotes(data: string): void {
  localStorage.setItem(AGENT_NOTES_KEY, data);
}
