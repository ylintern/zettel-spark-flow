import { useState } from "react";
import { RotateCcw } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { verifyVaultPassphrase } from "@/lib/commands";

interface ResetOnboardingSectionProps {
  hasPin: boolean;
}

export function ResetOnboardingSection({ hasPin }: ResetOnboardingSectionProps) {
  const [expanded, setExpanded] = useState(false);
  const [passphrase, setPassphrase] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [loading, setLoading] = useState(false);

  if (!hasPin) return null;

  const cancel = () => {
    setExpanded(false);
    setPassphrase("");
    setError(null);
    setSuccess(false);
  };

  const submit = async () => {
    setError(null);
    if (!passphrase) {
      setError("Passphrase is required to verify your identity.");
      return;
    }
    setLoading(true);
    try {
      // Non-destructive passphrase check. Previous implementation called
      // `reset_passphrase` with a garbage new value and relied on it failing
      // as a verifier — but `reset_passphrase` always mutates the vault
      // (deletes the old snapshot, recreates with the new garbage key),
      // which bricked unlock. `verify_vault_passphrase` only loads the
      // snapshot read-only and returns a bool, leaving the vault intact.
      const ok = await verifyVaultPassphrase(passphrase);
      if (!ok) {
        setError("Current passphrase is incorrect.");
        setLoading(false);
        return;
      }

      // Passphrase verified — safe to proceed with the onboarding reset.
      // Only onboarding.json is deleted; myspace/ (notes, tasks, folders)
      // and the vault snapshot are preserved.
      await invoke("reset_onboarding");
      setSuccess(true);
      setPassphrase("");
      setTimeout(() => {
        setExpanded(false);
        setSuccess(false);
        window.location.reload();
      }, 1500);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="card-3d rounded-2xl p-4">
      <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-1.5">
        <RotateCcw className="h-3.5 w-3.5" />
        Reset Onboarding
      </h2>

      {!expanded ? (
        <>
          <p className="text-[11px] text-muted-foreground mb-3 leading-relaxed">
            Re-run the setup wizard from the beginning. Your notes and vault data will be preserved.
          </p>
          <button
            onClick={() => setExpanded(true)}
            className="w-full rounded-lg border border-border text-sm py-2 text-foreground hover:bg-muted/40 transition-colors"
          >
            Reset onboarding
          </button>
        </>
      ) : (
        <div className="space-y-2">
          {success ? (
            <p className="text-xs text-primary font-medium py-1">Onboarding reset. Reloading…</p>
          ) : (
            <>
              <p className="text-[11px] text-muted-foreground mb-2">
                Enter your current passphrase to confirm, then the wizard will re-run on next launch.
              </p>
              <input
                type="password"
                placeholder="Current passphrase"
                value={passphrase}
                onChange={(e) => setPassphrase(e.target.value)}
                autoFocus
                className="w-full rounded-lg bg-background border border-border px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              />
              {error && <p className="text-xs text-destructive">{error}</p>}
              <div className="flex gap-2 pt-1">
                <button
                  onClick={submit}
                  disabled={loading}
                  className="flex-1 rounded-lg bg-primary text-primary-foreground text-sm py-2 font-medium hover:bg-primary/90 transition-colors disabled:opacity-50"
                >
                  {loading ? "Resetting…" : "Reset"}
                </button>
                <button
                  onClick={cancel}
                  disabled={loading}
                  className="flex-1 rounded-lg border border-border text-sm py-2 text-foreground hover:bg-muted/40 transition-colors disabled:opacity-50"
                >
                  Cancel
                </button>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}
