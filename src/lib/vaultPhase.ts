/**
 * Vault Phase Derivation — Single Source of Truth
 * 
 * This module exports the canonical logic for determining app phase
 * based on backend vault status. Nothing else should call setPhase directly.
 * All phase transitions are driven by the vault_status_changed event.
 */

export type VaultPhase = "onboarding" | "lock" | "app";

export interface VaultStatus {
  configured: boolean;
  unlocked: boolean;
  reason?: string;
}

/**
 * Derive the correct vault phase from backend status
 * 
 * Rules (locked):
 * - configured === false                    → "onboarding" (vault wiped, user restarts)
 * - configured === true  && unlocked === false → "lock"      (vault exists, user needs passphrase)
 * - configured === true  && unlocked === true  → "app"       (user in workspace)
 */
export function deriveVaultPhase(status: VaultStatus): VaultPhase {
  if (!status.configured) {
    return "onboarding";
  }
  if (!status.unlocked) {
    return "lock";
  }
  return "app";
}
