import { useStore } from "@/lib/store";
import { Moon, Sun, Trash2, Download, Shield, KeyRound } from "lucide-react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useState, useEffect } from "react";
import { isPinSetup } from "@/lib/crypto";
import { LocalModelsSection } from "@/components/settings/LocalModelsSection";
import { CloudProvidersSection } from "@/components/settings/CloudProvidersSection";
import { BiometricsSection } from "@/components/settings/BiometricsSection";
import { SafeVaultResetSection } from "@/components/settings/SafeVaultResetSection";
import { invoke } from "@tauri-apps/api/core";
import { exportNotes as exportNotesCmd, isTauriRuntimeAvailable } from "@/lib/commands";

const TOR_TOGGLE_KEY = "zettel-tor-enabled";

export function SettingsView() {
  const { notes } = useStore();
  const [dark, setDark] = useState(() => (localStorage.getItem("vibo_theme") ?? "dark") === "dark");
  const [torEnabled, setTorEnabled] = useState(() => localStorage.getItem(TOR_TOGGLE_KEY) === "true");
  const [hasPin, setHasPin] = useState(false);

  // Pass/PIN reset form state
  const [showResetForm, setShowResetForm] = useState(false);
  const [resetCurrent, setResetCurrent] = useState("");
  const [resetNew, setResetNew] = useState("");
  const [resetConfirm, setResetConfirm] = useState("");
  const [resetError, setResetError] = useState<string | null>(null);
  const [resetSuccess, setResetSuccess] = useState(false);
  const [resetLoading, setResetLoading] = useState(false);

  useEffect(() => {
    void isPinSetup().then(setHasPin).catch(() => setHasPin(false));
  }, []);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    localStorage.setItem("vibo_theme", dark ? "dark" : "light");
  }, [dark]);

  useEffect(() => {
    localStorage.setItem(TOR_TOGGLE_KEY, String(torEnabled));
  }, [torEnabled]);

  const handleResetPassphrase = async () => {
    setResetError(null);
    if (!resetCurrent || !resetNew || !resetConfirm) {
      setResetError("All fields are required.");
      return;
    }
    if (resetNew !== resetConfirm) {
      setResetError("New passphrase and confirmation do not match.");
      return;
    }
    setResetLoading(true);
    try {
      await invoke("reset_passphrase", {
        current: resetCurrent,
        newPassphrase: resetNew,
        caller: { type: "user" },
      });
      setResetSuccess(true);
      setResetCurrent("");
      setResetNew("");
      setResetConfirm("");
      setTimeout(() => {
        setShowResetForm(false);
        setResetSuccess(false);
      }, 2000);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setResetError(msg.includes("Invalid passphrase") ? "Current passphrase is incorrect." : msg);
    } finally {
      setResetLoading(false);
    }
  };

  const exportNotes = async () => {
    if (!isTauriRuntimeAvailable()) {
      // Fallback for non-Tauri (browser dev)
      const blob = new Blob([JSON.stringify(notes, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "zettelkasten-export.json";
      a.click();
      URL.revokeObjectURL(url);
      return;
    }
    try {
      const [json, { save }, { writeTextFile }] = await Promise.all([
        exportNotesCmd(),
        import("@tauri-apps/plugin-dialog"),
        import("@tauri-apps/plugin-fs"),
      ]);
      const filePath = await save({
        defaultPath: "zettelkasten-export.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (filePath) {
        await writeTextFile(filePath, json);
      }
    } catch (err) {
      console.error("Export failed:", err);
    }
  };

  const clearAll = () => {
    if (confirm("Delete all notes? This cannot be undone.")) {
      localStorage.removeItem("zettel-notes");
      localStorage.removeItem("zettel-encrypted-notes");
      window.location.reload();
    }
  };

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col gap-4 p-4 pb-20 max-w-md mx-auto">
        <h1 className="text-lg font-bold text-foreground">Settings</h1>

        {/* Appearance */}
        <div className="card-3d rounded-2xl p-4">
          <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-3">Appearance</h2>
          <button
            onClick={() => setDark(!dark)}
            className="w-full flex items-center justify-between py-2 text-sm text-foreground"
          >
            <span className="flex items-center gap-2">
              {dark ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
              {dark ? "Dark Mode" : "Light Mode"}
            </span>
            <div className={`w-10 h-5 rounded-full transition-colors ${dark ? "bg-foreground/30" : "bg-muted"} relative`}>
              <div className={`absolute top-0.5 h-4 w-4 rounded-full bg-foreground transition-transform ${dark ? "left-5" : "left-0.5"}`} />
            </div>
          </button>
        </div>

        {/* Encryption */}
        <div className="card-3d rounded-2xl p-4">
          <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-1.5">
            <Shield className="h-3.5 w-3.5" />
            Encryption
          </h2>
          <div className="space-y-3">
            <div className="flex items-center justify-between py-1">
              <div>
                <div className="text-sm font-medium text-foreground">AES-256-GCM</div>
                <div className="text-[10px] text-muted-foreground">App lock/unlock with PASS/PIN</div>
              </div>
              <div className={`flex items-center gap-1.5 text-xs font-medium ${hasPin ? "text-primary" : "text-muted-foreground"}`}>
                <div className={`h-2 w-2 rounded-full ${hasPin ? "bg-primary" : "bg-muted-foreground/40"}`} />
                {hasPin ? "Active" : "Not Set"}
              </div>
            </div>

            {hasPin && !showResetForm && (
              <button
                onClick={() => { setShowResetForm(true); setResetError(null); setResetSuccess(false); }}
                className="w-full flex items-center gap-2 py-2 text-sm text-foreground/70 hover:text-foreground transition-colors"
              >
                <KeyRound className="h-4 w-4" />
                Reset Pass / PIN
              </button>
            )}

            {showResetForm && (
              <div className="space-y-2 pt-1">
                {resetSuccess ? (
                  <p className="text-xs text-primary font-medium py-1">Passphrase updated successfully.</p>
                ) : (
                  <>
                    <input
                      type="password"
                      placeholder="Current passphrase"
                      value={resetCurrent}
                      onChange={e => setResetCurrent(e.target.value)}
                      className="w-full rounded-lg bg-background border border-border px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
                    />
                    <input
                      type="password"
                      placeholder="New passphrase"
                      value={resetNew}
                      onChange={e => setResetNew(e.target.value)}
                      className="w-full rounded-lg bg-background border border-border px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
                    />
                    <input
                      type="password"
                      placeholder="Confirm new passphrase"
                      value={resetConfirm}
                      onChange={e => setResetConfirm(e.target.value)}
                      className="w-full rounded-lg bg-background border border-border px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
                    />
                    {resetError && (
                      <p className="text-xs text-destructive">{resetError}</p>
                    )}
                    <div className="flex gap-2 pt-1">
                      <button
                        onClick={handleResetPassphrase}
                        disabled={resetLoading}
                        className="flex-1 rounded-lg bg-primary text-primary-foreground text-sm py-2 font-medium hover:bg-primary/90 transition-colors disabled:opacity-50"
                      >
                        {resetLoading ? "Updating…" : "Update"}
                      </button>
                      <button
                        onClick={() => { setShowResetForm(false); setResetError(null); setResetCurrent(""); setResetNew(""); setResetConfirm(""); }}
                        disabled={resetLoading}
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
        </div>

        {/* Biometrics (Mobile only) */}
        <BiometricsSection />

        {/* Local Models */}
        <LocalModelsSection />

        {/* Cloud Providers */}
        <CloudProvidersSection
          torEnabled={torEnabled}
          onTorToggle={() => setTorEnabled(!torEnabled)}
        />

        {/* Data */}
        <div className="card-3d rounded-2xl p-4">
          <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-3">Data</h2>
          <div className="space-y-2">
            <button onClick={() => { void exportNotes(); }} className="w-full flex items-center gap-2 py-2 text-sm text-foreground hover:text-foreground/80 transition-colors">
              <Download className="h-4 w-4" />
              Export Notes as JSON
            </button>
            <button onClick={clearAll} className="w-full flex items-center gap-2 py-2 text-sm text-destructive hover:text-destructive/80 transition-colors">
              <Trash2 className="h-4 w-4" />
              Delete All Notes
            </button>
          </div>
        </div>

        {/* Safe Vault Reset */}
        <SafeVaultResetSection />

        {/* Stats */}
        <div className="card-3d rounded-2xl p-4">
          <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-3">Stats</h2>
          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <div className="text-lg font-bold text-foreground">{notes.length}</div>
              <div className="text-[10px] text-muted-foreground uppercase">Total Notes</div>
            </div>
            <div>
              <div className="text-lg font-bold text-foreground">{notes.filter(n => n.isKanban).length}</div>
              <div className="text-[10px] text-muted-foreground uppercase">Tasks</div>
            </div>
          </div>
        </div>
      </div>
    </ScrollArea>
  );
}
