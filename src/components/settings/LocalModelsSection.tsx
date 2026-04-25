import { useEffect, useMemo, useRef, useState } from "react";
import { Cpu, Download, Trash2, Loader2, ChevronDown } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  listModels,
  listDownloaded,
  getActiveModel,
  setActiveModel,
  downloadModel,
  deleteModel,
  onModelDownloadProgress,
  onModelState,
  type ModelEntryDto,
} from "@/lib/inference";

/**
 * LocalModelsSection — Settings UI for on-device LFM models.
 *
 * Source of truth is the Rust `InferenceService` (via `@/lib/inference`).
 * Three states per model:
 *   - active     → currently loaded and selected for chat
 *   - downloaded → cached on disk, ready to activate
 *   - available  → not yet downloaded
 *
 * Cross-install discovery is automatic: `listDownloaded()` reflects the
 * leap-ai plugin's actual on-disk cache, so any whitelisted LFM already
 * pulled by another LEAP-based app shows up as "downloaded" on first launch.
 */

type Status = "active" | "downloaded" | "available";

function StatusDot({ status }: { status: Status }) {
  if (status === "active") {
    return (
      <span
        title="Active"
        className="h-2 w-2 rounded-full bg-primary shadow-[0_0_6px_hsl(var(--primary))]"
      />
    );
  }
  if (status === "downloaded") {
    return (
      <span
        title="Downloaded"
        className="h-2 w-2 rounded-full bg-muted-foreground/60"
      />
    );
  }
  return (
    <span
      title="Not downloaded"
      className="h-2 w-2 rounded-full border border-muted-foreground/40"
    />
  );
}

function ModelCard({
  model,
  status,
  downloading,
  progress,
  onDownload,
  onActivate,
  onDelete,
}: {
  model: ModelEntryDto;
  status: Status;
  downloading: boolean;
  progress: number;
  onDownload: () => void;
  onActivate: () => void;
  onDelete: () => void;
}) {
  const renderAction = () => {
    if (downloading) {
      return <Loader2 className="h-4 w-4 text-primary animate-spin" />;
    }
    if (status === "active") {
      return (
        <button
          onClick={onDelete}
          title="Remove from device"
          className="p-1 text-muted-foreground hover:text-destructive transition-colors"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      );
    }
    if (status === "downloaded") {
      return (
        <div className="flex items-center gap-1">
          <button
            onClick={onActivate}
            className="text-[10px] font-medium text-primary hover:text-primary/80 px-2 py-1 rounded-lg bg-primary/10 hover:bg-primary/20 transition-colors"
          >
            Activate
          </button>
          <button
            onClick={onDelete}
            title="Remove from device"
            className="p-1 text-muted-foreground hover:text-destructive transition-colors"
          >
            <Trash2 className="h-3 w-3" />
          </button>
        </div>
      );
    }
    return (
      <button
        onClick={onDownload}
        className="flex items-center gap-1 text-[10px] font-medium text-foreground hover:text-primary px-2 py-1 rounded-lg bg-muted hover:bg-primary/10 transition-colors"
      >
        <Download className="h-3 w-3" /> Pull
      </button>
    );
  };

  return (
    <div
      className={`rounded-xl border p-3 space-y-2 transition-colors ${
        status === "active"
          ? "border-primary bg-primary/5"
          : "border-border bg-muted/30"
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5">
            <StatusDot status={status} />
            <span className="text-sm font-semibold text-foreground truncate">
              {model.name}
            </span>
            {model.alias && (
              <span className="text-[9px] font-mono lowercase text-muted-foreground/80 px-1.5 py-0.5 rounded-md bg-muted/50">
                {model.alias}
              </span>
            )}
            {model.recommended && (
              <span className="text-[8px] font-bold uppercase tracking-wider bg-primary/20 text-primary px-1.5 py-0.5 rounded-full">
                rec
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 mt-0.5 pl-3.5">
            <span className="text-[10px] font-mono text-muted-foreground">
              {model.params}
            </span>
            <span className="text-[10px] text-muted-foreground">•</span>
            <span className="text-[10px] text-muted-foreground">
              {model.modality.join(" + ")}
            </span>
            <span className="text-[10px] text-muted-foreground">•</span>
            <span className="text-[10px] text-muted-foreground">
              {model.sizeMb}MB
            </span>
          </div>
        </div>
        {renderAction()}
      </div>
      <p className="text-[10px] text-muted-foreground leading-relaxed">
        {model.description}
      </p>
      {downloading && (
        <div className="space-y-1">
          <Progress value={Math.min(progress, 100)} className="h-1.5" />
          <div className="text-[9px] text-muted-foreground text-right">
            {Math.min(Math.round(progress), 100)}%
          </div>
        </div>
      )}
    </div>
  );
}

export function LocalModelsSection() {
  const [models, setModels] = useState<ModelEntryDto[]>([]);
  const [downloaded, setDownloaded] = useState<Set<string>>(new Set());
  const [activeId, setActiveId] = useState<string | null>(null);
  const [progressById, setProgressById] = useState<Record<string, number>>({});
  const [downloadingIds, setDownloadingIds] = useState<Set<string>>(new Set());
  const [open, setOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Track which plugin labels map to which internal id (for download-progress event filtering).
  const labelMap = useRef<Record<string, string>>({});

  const refreshState = async () => {
    try {
      const [dl, active] = await Promise.all([
        listDownloaded(),
        getActiveModel(),
      ]);
      setDownloaded(new Set(dl));
      setActiveId(active);
    } catch (e) {
      console.error("[LocalModels] refreshState failed:", e);
    }
  };

  useEffect(() => {
    let cancelled = false;
    let unlistenProgress: (() => void) | null = null;
    let unlistenState: (() => void) | null = null;

    (async () => {
      try {
        const list = await listModels();
        if (cancelled) return;
        setModels(list);
        list.forEach((m) => {
          // Best-effort label cache for progress event filtering.
          // (Rust events emit the internal id already, but keep the map for safety.)
          labelMap.current[m.id] = m.id;
        });
        await refreshState();

        unlistenProgress = await onModelDownloadProgress((ev) => {
          // ev.modelId is the plugin label; we may receive internal id too.
          // Match by id-substring as a defensive fallback.
          const id =
            list.find(
              (m) => ev.modelId === m.id || ev.modelId.includes(m.id),
            )?.id ?? ev.modelId;
          setProgressById((prev) => ({
            ...prev,
            [id]: Math.round(ev.progress * 100),
          }));
        });

        unlistenState = await onModelState((ev) => {
          // Any state transition → refetch the truth.
          void refreshState();
          if (
            ev.state === "model-loaded" ||
            ev.state === "download-removed"
          ) {
            // Clear progress for this id once the download/load finishes.
            const id =
              list.find(
                (m) => ev.modelId === m.id || ev.modelId.includes(m.id),
              )?.id ?? ev.modelId;
            setDownloadingIds((prev) => {
              const next = new Set(prev);
              next.delete(id);
              return next;
            });
            setProgressById((prev) => {
              const next = { ...prev };
              delete next[id];
              return next;
            });
          }
        });
      } catch (e) {
        console.error("[LocalModels] mount failed:", e);
        if (!cancelled) setError(String(e));
      }
    })();

    return () => {
      cancelled = true;
      unlistenProgress?.();
      unlistenState?.();
    };
  }, []);

  const statusFor = (id: string): Status => {
    if (activeId === id) return "active";
    if (downloaded.has(id)) return "downloaded";
    return "available";
  };

  const handleDownload = async (id: string) => {
    setError(null);
    setDownloadingIds((prev) => new Set(prev).add(id));
    setProgressById((prev) => ({ ...prev, [id]: 0 }));
    try {
      await downloadModel(id);
      // The plugin's auto-load on download completion does NOT emit a
      // `model-loaded` event (only explicit `load_model` calls do — see
      // tauri-plugin-leap-ai-0.1.1/src/desktop.rs:165). So we cannot rely
      // on `onModelState` to clear the spinner — clear it here on the
      // resolution of `downloadModel`.
      await refreshState();
    } catch (e) {
      console.error("[LocalModels] download failed:", e);
      setError(`Download failed: ${e}`);
    } finally {
      setDownloadingIds((prev) => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
      setProgressById((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
    }
  };

  const handleActivate = async (id: string) => {
    setError(null);
    try {
      await setActiveModel(id);
      await refreshState();
    } catch (e) {
      console.error("[LocalModels] activate failed:", e);
      setError(`Activate failed: ${e}`);
    }
  };

  const handleDelete = async (id: string) => {
    setError(null);
    try {
      await deleteModel(id);
      await refreshState();
    } catch (e) {
      console.error("[LocalModels] delete failed:", e);
      setError(`Delete failed: ${e}`);
    }
  };

  const installed = useMemo(
    () => models.filter((m) => downloaded.has(m.id) || activeId === m.id),
    [models, downloaded, activeId],
  );
  const available = useMemo(
    () => models.filter((m) => !downloaded.has(m.id) && activeId !== m.id),
    [models, downloaded, activeId],
  );

  return (
    <div className="card-3d rounded-2xl p-4">
      <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-1 flex items-center gap-1.5">
        <Cpu className="h-3.5 w-3.5" />
        Local Models — Liquid AI
      </h2>
      <p className="text-[10px] text-muted-foreground mb-3">
        LFM 2.5 nano agents run privately on-device via LEAP.
        {models.length > 0 ? ` ${models.length} models available.` : ""}
      </p>

      {error && (
        <div className="text-[10px] text-destructive bg-destructive/10 border border-destructive/30 rounded-lg p-2 mb-2">
          {error}
        </div>
      )}

      {/* Installed (downloaded or active) */}
      <div className="space-y-2 mb-3">
        {installed.map((model) => (
          <ModelCard
            key={model.id}
            model={model}
            status={statusFor(model.id)}
            downloading={downloadingIds.has(model.id)}
            progress={progressById[model.id] ?? 0}
            onDownload={() => handleDownload(model.id)}
            onActivate={() => handleActivate(model.id)}
            onDelete={() => handleDelete(model.id)}
          />
        ))}
        {installed.length === 0 && models.length > 0 && (
          <p className="text-[10px] text-muted-foreground italic">
            No models installed yet. Pull one below to get started.
          </p>
        )}
      </div>

      {/* Available to download */}
      {available.length > 0 && (
        <Collapsible open={open} onOpenChange={setOpen}>
          <CollapsibleTrigger className="w-full flex items-center justify-between py-2 text-[11px] font-medium text-muted-foreground hover:text-foreground transition-colors">
            <span>
              {available.length} more Liquid AI{" "}
              {available.length === 1 ? "model" : "models"} available
            </span>
            <ChevronDown
              className={`h-3.5 w-3.5 transition-transform duration-200 ${
                open ? "rotate-180" : ""
              }`}
            />
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="space-y-2 pt-1">
              {available.map((model) => (
                <ModelCard
                  key={model.id}
                  model={model}
                  status="available"
                  downloading={downloadingIds.has(model.id)}
                  progress={progressById[model.id] ?? 0}
                  onDownload={() => handleDownload(model.id)}
                  onActivate={() => handleActivate(model.id)}
                  onDelete={() => handleDelete(model.id)}
                />
              ))}
            </div>
          </CollapsibleContent>
        </Collapsible>
      )}
    </div>
  );
}
