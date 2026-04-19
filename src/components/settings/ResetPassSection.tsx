import { useState } from "react";
import { KeyRound } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface ResetPassSectionProps {
  hasPin: boolean;
}

export function ResetPassSection({ hasPin }: ResetPassSectionProps) {
  const [expanded, setExpanded] = useState(false);
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [loading, setLoading] = useState(false);

  if (!hasPin) return null;

  const reset = () => {
    setCurrent("");
    setNext("");
    setConfirm("");
    setError(null);
    setSuccess(false);
    setLoading(false);
  };

  const cancel = () => {
    setExpanded(false);
    reset();
  };

  const submit = async () => {
    setError(null);
    if (!current || !next || !confirm) {
      setError("All fields are required.");
      return;
    }
    if (next !== confirm) {
      setError("New passphrase and confirmation do not match.");
      return;
    }
    if (next === current) {
      setError("New passphrase must differ from the current one.");
      return;
    }
    setLoading(true);
    try {
      await invoke("reset_passphrase", {
        current,
        newPassphrase: next,
        caller: { type: "user" },
      });
      setSuccess(true);
      setCurrent("");
      setNext("");
      setConfirm("");
      setTimeout(() => {
        setExpanded(false);
        setSuccess(false);
      }, 2000);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg.includes("Invalid passphrase") ? "Current passphrase is incorrect." : msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="card-3d rounded-2xl p-4">
      <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-1.5">
        <KeyRound className="h-3.5 w-3.5" />
        Reset Passphrase
      </h2>

      {!expanded ? (
        <>
          <p className="text-[11px] text-muted-foreground mb-3 leading-relaxed">
            Change the passphrase used to lock and unlock the app. Your notes and settings stay intact.
          </p>
          <button
            onClick={() => { setExpanded(true); reset(); }}
            className="w-full rounded-lg border border-border text-sm py-2 text-foreground hover:bg-muted/40 transition-colors"
          >
            Change passphrase
          </button>
        </>
      ) : (
        <div className="space-y-2">
          {success ? (
            <p className="text-xs text-primary font-medium py-1">Passphrase updated successfully.</p>
          ) : (
            <>
              <input
                type="password"
                placeholder="Current passphrase"
                value={current}
                onChange={(e) => setCurrent(e.target.value)}
                autoFocus
                className="w-full rounded-lg bg-background border border-border px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              />
              <input
                type="password"
                placeholder="New passphrase"
                value={next}
                onChange={(e) => setNext(e.target.value)}
                className="w-full rounded-lg bg-background border border-border px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              />
              <input
                type="password"
                placeholder="Confirm new passphrase"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") void submit(); }}
                className="w-full rounded-lg bg-background border border-border px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              />
              {error && <p className="text-xs text-destructive">{error}</p>}
              <div className="flex gap-2 pt-1">
                <button
                  onClick={submit}
                  disabled={loading}
                  className="flex-1 rounded-lg bg-primary text-primary-foreground text-sm py-2 font-medium hover:bg-primary/90 transition-colors disabled:opacity-50"
                >
                  {loading ? "Updating…" : "Update"}
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
