import { useEffect, useState } from "react";
import { Cloud, Key, Server, Globe, Check, X, ChevronDown, Unplug } from "lucide-react";
import {
  CLOUD_PROVIDERS,
  getCloudKeys,
  loadCloudKeys,
  setCloudKey,
  getActiveProvider,
  setActiveProvider,
  getSelectedModels,
  setSelectedModel,
  type CloudProviderType,
} from "@/lib/models";

function ModelSelector({ provider, selectedModel, onSelect }: {
  provider: typeof CLOUD_PROVIDERS[number];
  selectedModel: string;
  onSelect: (model: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const models = provider.models || [];
  if (models.length === 0) return null;

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="w-full h-8 rounded-lg bg-muted border border-border px-3 text-xs text-foreground flex items-center justify-between hover:border-muted-foreground/30 transition-colors"
      >
        <span className={selectedModel ? "text-foreground" : "text-muted-foreground"}>
          {selectedModel || "Select a model…"}
        </span>
        <ChevronDown className={`h-3 w-3 text-muted-foreground transition-transform ${open ? "rotate-180" : ""}`} />
      </button>
      {open && (
        <div className="absolute top-9 left-0 right-0 z-50 rounded-lg border border-border bg-card shadow-lg max-h-48 overflow-y-auto">
          {models.map((m) => (
            <button
              key={m}
              onClick={() => { onSelect(m); setOpen(false); }}
              className={`w-full text-left px-3 py-2 text-xs hover:bg-muted transition-colors ${
                selectedModel === m ? "bg-primary/10 text-primary font-medium" : "text-foreground"
              }`}
            >
              {m}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function ProviderCard({
  provider,
  connected,
  active,
  selectedModel,
  onConnect,
  onDisconnect,
  onActivate,
  onModelSelect,
}: {
  provider: typeof CLOUD_PROVIDERS[number];
  connected: boolean;
  active: boolean;
  selectedModel: string;
  onConnect: (key: string) => void;
  onDisconnect: () => void;
  onActivate: () => void;
  onModelSelect: (model: string) => void;
}) {
  const [connecting, setConnecting] = useState(false);
  const [keyInput, setKeyInput] = useState("");

  const handleConnect = () => {
    if (keyInput.trim()) {
      onConnect(keyInput.trim());
      setKeyInput("");
      setConnecting(false);
    }
  };

  return (
    <div className={`rounded-xl border p-3 space-y-2 transition-colors ${
      active ? "border-primary bg-primary/5" : connected ? "border-border bg-accent/30" : "border-border bg-muted/30"
    }`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className={`h-7 w-7 rounded-lg flex items-center justify-center ${
            connected ? "bg-primary/10" : "bg-muted"
          }`}>
            {provider.type === "host" ? <Server className="h-3.5 w-3.5 text-muted-foreground" /> : <Key className="h-3.5 w-3.5 text-muted-foreground" />}
          </div>
          <div>
            <div className="text-sm font-semibold text-foreground">{provider.name}</div>
            <div className="text-[10px] text-muted-foreground">{provider.description}</div>
          </div>
        </div>
        {connected ? (
          <div className="flex items-center gap-1.5">
            {!active && (
              <button
                onClick={onActivate}
                className="text-[10px] font-medium text-primary hover:text-primary/80 px-2 py-1 rounded-lg bg-primary/10 hover:bg-primary/20 transition-colors"
              >
                Use
              </button>
            )}
            {active && (
              <span className="text-[10px] font-medium text-primary flex items-center gap-1">
                <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                Active
              </span>
            )}
          </div>
        ) : (
          <button
            onClick={() => setConnecting(true)}
            className="text-[10px] font-semibold text-foreground bg-foreground/10 hover:bg-foreground/20 px-3 py-1.5 rounded-lg transition-colors"
          >
            Connect
          </button>
        )}
      </div>

      {/* Connecting flow — inline key input */}
      {connecting && !connected && (
        <div className="flex gap-2 items-center">
          <input
            type="password"
            value={keyInput}
            onChange={(e) => setKeyInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleConnect()}
            placeholder={provider.placeholder}
            className="flex-1 h-8 rounded-lg bg-muted border border-border px-3 text-xs font-mono text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary/30"
            autoFocus
          />
          <button onClick={handleConnect} className="h-8 w-8 rounded-lg bg-primary text-primary-foreground flex items-center justify-center hover:opacity-90 transition-opacity">
            <Check className="h-3.5 w-3.5" />
          </button>
          <button onClick={() => { setConnecting(false); setKeyInput(""); }} className="h-8 w-8 rounded-lg bg-muted text-muted-foreground flex items-center justify-center hover:text-foreground transition-colors">
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      )}

      {/* Connected state — model selector + disconnect */}
      {connected && (
        <div className="space-y-2">
          {provider.models && provider.models.length > 0 && (
            <ModelSelector
              provider={provider}
              selectedModel={selectedModel}
              onSelect={onModelSelect}
            />
          )}
          <button
            onClick={onDisconnect}
            className="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-destructive transition-colors"
          >
            <Unplug className="h-3 w-3" />
            Disconnect
          </button>
        </div>
      )}
    </div>
  );
}

export function CloudProvidersSection({
  torEnabled,
  onTorToggle,
}: {
  torEnabled: boolean;
  onTorToggle: () => void;
}) {
  const [keys, setKeys] = useState(getCloudKeys);
  const [activeProvider, setActive] = useState(getActiveProvider);
  const [selectedModels, setModels] = useState(getSelectedModels);

  useEffect(() => {
    void loadCloudKeys().then(setKeys).catch((error) => {
      console.error("Failed to load secure cloud keys:", error);
    });
  }, []);

  const handleConnect = async (provider: CloudProviderType, key: string) => {
    await setCloudKey(provider, key);
    setKeys(getCloudKeys());
    if (activeProvider === "local") {
      setActiveProvider(provider);
      setActive(provider);
    }
  };

  const handleDisconnect = async (provider: CloudProviderType) => {
    await setCloudKey(provider, "");
    setKeys(getCloudKeys());
    if (activeProvider === provider) {
      setActiveProvider("local");
      setActive("local");
    }
  };

  const activate = (provider: CloudProviderType) => {
    setActiveProvider(provider);
    setActive(provider);
  };

  const handleModelSelect = (provider: CloudProviderType, model: string) => {
    setSelectedModel(provider, model);
    setModels(getSelectedModels());
  };

  const visibleProviders = CLOUD_PROVIDERS.filter((p) => !p.hidden);

  return (
    <div className="card-3d rounded-2xl p-4">
      <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-1 flex items-center gap-1.5">
        <Cloud className="h-3.5 w-3.5" />
        Cloud Providers
      </h2>
      <p className="text-[10px] text-muted-foreground mb-3">
        Connect cloud providers for deep reasoning. Keys are stored in the secure vault.
      </p>

      <div className="space-y-2">
        {visibleProviders.map((provider) => (
          <ProviderCard
            key={provider.id}
            provider={provider}
            connected={!!keys[provider.id]}
            active={activeProvider === provider.id}
            selectedModel={selectedModels[provider.id] || ""}
            onConnect={(key) => { void handleConnect(provider.id, key); }}
            onDisconnect={() => { void handleDisconnect(provider.id); }}
            onActivate={() => activate(provider.id)}
            onModelSelect={(m) => handleModelSelect(provider.id, m)}
          />
        ))}
      </div>

      {/* Tor Toggle */}
      <div className="mt-3 pt-3 border-t border-border">
        <button
          onClick={onTorToggle}
          className="w-full flex items-center justify-between py-1.5 text-sm text-foreground"
        >
          <span className="flex items-center gap-2">
            <Globe className="h-4 w-4" />
            <div className="text-left">
              <div className="text-xs font-medium">Tor Routing</div>
              <div className="text-[9px] text-muted-foreground">Route cloud API calls through Tor network</div>
            </div>
          </span>
          <div className={`w-9 h-5 rounded-full transition-colors ${torEnabled ? "bg-primary/60" : "bg-muted"} relative`}>
            <div className={`absolute top-0.5 h-4 w-4 rounded-full transition-transform ${torEnabled ? "bg-primary left-[18px]" : "bg-foreground left-0.5"}`} />
          </div>
        </button>
      </div>
    </div>
  );
}
