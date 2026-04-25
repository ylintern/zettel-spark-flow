import { useState, useRef, useEffect, useCallback } from "react";
import { useStore } from "@/lib/store";
import { Send, FileText, Columns3, Bot, User, Zap, ZapOff, ChevronDown } from "lucide-react";
import {
  isLfmConfigured,
  refreshLocalReady,
  streamLfmChat,
  getActiveProviderLabel,
  type LfmMessage,
} from "@/lib/lfm";
import {
  CLOUD_PROVIDERS,
  getActiveProvider,
  setActiveProvider,
  getCloudKeys,
  loadCloudKeys,
  getSelectedModels,
  setSelectedModel,
  type ActiveProvider,
  type CloudProviderType,
} from "@/lib/models";
import {
  endChatSession,
  getActiveModel,
  getSessionInfo,
  listDownloaded,
  listModels,
  setActiveModel,
  type ModelEntryDto,
  type SessionInfo,
} from "@/lib/inference";

interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
}

const GREETING: ChatMessage = {
  id: "welcome",
  role: "assistant",
  content: "Hey! I can help you create notes, tasks, or just chat. Try:\n• **\"new note: My Idea\"** to create a note\n• **\"new task: Fix bug\"** to create a task\n• Or just ask me anything!",
};

const sessionMessages = new Map<string, ChatMessage[]>();

interface ModelSwitcherProps {
  localModels: ModelEntryDto[];
  downloadedIds: Set<string>;
  activeLocalId: string | null;
  onLocalSelect: (id: string) => void;
}

function ModelSwitcher({
  localModels,
  downloadedIds,
  activeLocalId,
  onLocalSelect,
}: ModelSwitcherProps) {
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState<ActiveProvider>(getActiveProvider);
  const [selModels, setSelModels] = useState(getSelectedModels);
  const keys = getCloudKeys();

  useEffect(() => {
    void loadCloudKeys().then(() => {
      setSelModels(getSelectedModels());
    }).catch((error) => {
      console.error("Failed to load secure cloud keys:", error);
    });
  }, []);

  const availableProviders = CLOUD_PROVIDERS.filter((p) => keys[p.id]);
  const currentProvider = active === "local" ? null : CLOUD_PROVIDERS.find((p) => p.id === active);
  const currentModel = active !== "local" ? selModels[active as CloudProviderType] : "";
  const localActive = localModels.find((m) => m.id === activeLocalId);
  const displayLabel = active === "local"
    ? localActive ? `Local · ${localActive.alias}` : "Local LFM"
    : `${currentProvider?.name || active}${currentModel ? " / " + currentModel : ""}`;

  const downloadedLocal = localModels.filter((m) => downloadedIds.has(m.id));

  const handleCloudSelect = (provider: ActiveProvider, model?: string) => {
    setActiveProvider(provider);
    setActive(provider);
    if (model && provider !== "local") {
      setSelectedModel(provider as CloudProviderType, model);
      setSelModels(getSelectedModels());
    }
    setOpen(false);
  };

  const handleLocalRowClick = (id: string) => {
    setActiveProvider("local");
    setActive("local");
    onLocalSelect(id);
    setOpen(false);
  };

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1 text-[9px] font-mono text-muted-foreground hover:text-foreground transition-colors px-1.5 py-0.5 rounded-md hover:bg-accent max-w-[160px]"
      >
        {isLfmConfigured() ? <Zap className="h-2.5 w-2.5 text-primary shrink-0" /> : <ZapOff className="h-2.5 w-2.5 shrink-0" />}
        <span className="truncate">{displayLabel}</span>
        <ChevronDown className={`h-2.5 w-2.5 shrink-0 transition-transform ${open ? "rotate-180" : ""}`} />
      </button>
      {open && (
        <div className="absolute bottom-full right-0 mb-1 z-50 w-56 rounded-lg border border-border bg-card shadow-lg max-h-72 overflow-y-auto">
          {/* Local LFM — dynamic list of downloaded models */}
          <div className="px-3 py-1.5 text-[9px] font-bold uppercase text-muted-foreground tracking-wider bg-muted/50">Local</div>
          {downloadedLocal.length === 0 ? (
            <div className="px-3 py-2 text-[10px] text-muted-foreground">
              Pull a model in Settings → Local Models
            </div>
          ) : (
            downloadedLocal.map((m) => {
              const isActive = active === "local" && activeLocalId === m.id;
              return (
                <button
                  key={m.id}
                  onClick={() => handleLocalRowClick(m.id)}
                  className={`w-full text-left px-3 py-2 text-xs hover:bg-muted transition-colors flex items-center justify-between gap-2 ${
                    isActive ? "bg-primary/10 text-primary font-medium" : "text-foreground"
                  }`}
                >
                  <span className="truncate">{m.name}</span>
                  <span className="text-[9px] font-mono lowercase text-muted-foreground/80 px-1.5 py-0.5 rounded-md bg-muted/50 shrink-0">
                    {m.alias}
                  </span>
                </button>
              );
            })
          )}

          {/* Connected cloud providers grouped */}
          {availableProviders.map((p) => (
            <div key={p.id}>
              <div className="px-3 py-1.5 text-[9px] font-bold uppercase text-muted-foreground tracking-wider bg-muted/50 border-t border-border">
                {p.name}
              </div>
              {(p.models || []).length > 0 ? (
                (p.models || []).map((m) => (
                  <button
                    key={m}
                    onClick={() => handleCloudSelect(p.id, m)}
                    className={`w-full text-left px-3 py-1.5 text-[11px] hover:bg-muted transition-colors ${
                      active === p.id && selModels[p.id] === m ? "bg-primary/10 text-primary font-medium" : "text-foreground"
                    }`}
                  >
                    {m}
                  </button>
                ))
              ) : (
                <button
                  onClick={() => handleCloudSelect(p.id)}
                  className={`w-full text-left px-3 py-1.5 text-[11px] hover:bg-muted transition-colors ${
                    active === p.id ? "bg-primary/10 text-primary font-medium" : "text-foreground"
                  }`}
                >
                  Default
                </button>
              )}
            </div>
          ))}

          {availableProviders.length === 0 && (
            <div className="px-3 py-2 text-[10px] text-muted-foreground border-t border-border">
              Connect providers in Settings
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function fmtK(n: number): string {
  if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, "") + "k";
  return String(n);
}

export function ChatAssistant({ compact = false, sessionId = "default" }: { compact?: boolean; sessionId?: string }) {
  const { addNote, setActiveView } = useStore();

  if (!sessionMessages.has(sessionId)) {
    sessionMessages.set(sessionId, [GREETING]);
  }

  const [messages, setMessages] = useState<ChatMessage[]>(sessionMessages.get(sessionId)!);
  const [input, setInput] = useState("");
  const [isTyping, setIsTyping] = useState(false);
  const [lfmActive, setLfmActive] = useState(isLfmConfigured);
  const [providerLabel, setProviderLabel] = useState(getActiveProviderLabel);
  const [localModels, setLocalModels] = useState<ModelEntryDto[]>([]);
  const [downloadedIds, setDownloadedIds] = useState<Set<string>>(new Set());
  const [activeLocalId, setActiveLocalId] = useState<string | null>(null);
  const [sessionInfo, setSessionInfo] = useState<SessionInfo | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const abortRef = useRef<AbortController | null>(null);
  const localSessionRef = useRef<string | null>(null);

  const refreshSessionInfo = useCallback(async () => {
    const sid = localSessionRef.current;
    if (!sid) { setSessionInfo(null); return; }
    try {
      const info = await getSessionInfo(sid);
      setSessionInfo(info);
    } catch {
      // non-fatal — chip just stays blank
    }
  }, []);

  // Tear down the local llama session, if any.
  const teardownLocalSession = useCallback(async () => {
    const sid = localSessionRef.current;
    if (!sid) return;
    localSessionRef.current = null;
    setSessionInfo(null);
    try {
      await endChatSession(sid);
    } catch (e) {
      console.warn("[ChatAssistant] endChatSession failed:", e);
    }
  }, []);

  useEffect(() => {
    if (!sessionMessages.has(sessionId)) {
      sessionMessages.set(sessionId, [GREETING]);
    }
    setMessages(sessionMessages.get(sessionId)!);
    setInput("");
    setIsTyping(false);
    if (abortRef.current) {
      abortRef.current.abort();
      abortRef.current = null;
    }
    void teardownLocalSession();
  }, [sessionId, teardownLocalSession]);

  // On unmount: abort + tear down local session.
  useEffect(() => {
    return () => {
      if (abortRef.current) abortRef.current.abort();
      void teardownLocalSession();
    };
  }, [teardownLocalSession]);

  useEffect(() => { sessionMessages.set(sessionId, messages); }, [messages, sessionId]);
  useEffect(() => { if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight; }, [messages, isTyping]);

  // Refresh provider/model state every 2s. Picks up changes made in Settings
  // (download finished, new model activated, etc.) without needing wiring.
  useEffect(() => {
    let cancelled = false;
    const refresh = async () => {
      await loadCloudKeys().catch(() => undefined);
      await refreshLocalReady();
      if (cancelled) return;
      try {
        const [list, dl, active] = await Promise.all([
          listModels(),
          listDownloaded(),
          getActiveModel(),
        ]);
        if (cancelled) return;
        setLocalModels(list);
        setDownloadedIds(new Set(dl));
        // If the active model changed under us, drop the stale plugin session.
        if (active !== activeLocalId) {
          void teardownLocalSession();
          setActiveLocalId(active);
        }
      } catch (e) {
        console.error("[ChatAssistant] local model refresh failed:", e);
      }
      if (cancelled) return;
      setLfmActive(isLfmConfigured());
      setProviderLabel(getActiveProviderLabel());
    };
    void refresh();
    const iv = setInterval(() => { void refresh(); }, 2000);
    return () => { cancelled = true; clearInterval(iv); };
  }, [activeLocalId, teardownLocalSession]);

  const handleLocalSelect = useCallback(async (id: string) => {
    try {
      await setActiveModel(id);
      // Drop stale session — next send creates a fresh one bound to the new model.
      await teardownLocalSession();
      setActiveLocalId(id);
      await refreshLocalReady();
      setLfmActive(isLfmConfigured());
      setProviderLabel(getActiveProviderLabel());
    } catch (e) {
      console.error("[ChatAssistant] setActiveModel failed:", e);
    }
  }, [teardownLocalSession]);

  const handleCommand = useCallback((text: string): ChatMessage | null => {
    const lower = text.toLowerCase().trim();
    if (lower.startsWith("new note:") || lower.startsWith("note:")) {
      const title = text.replace(/^(new\s+)?note:\s*/i, "").trim() || "Untitled";
      addNote("inbox", false);
      setActiveView("notebook");
      return { id: crypto.randomUUID(), role: "assistant", content: `📝 Created note **"${title}"**. Switched to Notes view!` };
    }
    if (lower.startsWith("new task:") || lower.startsWith("task:")) {
      const title = text.replace(/^(new\s+)?task:\s*/i, "").trim() || "Untitled Task";
      addNote("inbox", true);
      setActiveView("kanban");
      return { id: crypto.randomUUID(), role: "assistant", content: `✅ Created task **"${title}"** in Inbox column!` };
    }
    return null;
  }, [addNote, setActiveView]);

  const handleSend = useCallback(() => {
    const text = input.trim();
    if (!text) return;
    const userMsg: ChatMessage = { id: crypto.randomUUID(), role: "user", content: text };
    setMessages((prev) => [...prev, userMsg]);
    setInput("");

    const cmdReply = handleCommand(text);
    if (cmdReply) {
      setIsTyping(true);
      setTimeout(() => { setMessages((prev) => [...prev, cmdReply]); setIsTyping(false); }, 300);
      return;
    }

    if (isLfmConfigured()) {
      setIsTyping(true);
      const assistantId = crypto.randomUUID();
      let assistantContent = "";
      const lfmMessages: LfmMessage[] = [
        ...messages.filter((m) => m.id !== "welcome").map((m) => ({ role: m.role as "user" | "assistant", content: m.content })),
        { role: "user" as const, content: text },
      ];
      const controller = new AbortController();
      abortRef.current = controller;
      void streamLfmChat({
        messages: lfmMessages,
        signal: controller.signal,
        localSessionRef,
        onDelta: (chunk) => {
          assistantContent += chunk;
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            if (last?.id === assistantId) return prev.map((m) => (m.id === assistantId ? { ...m, content: assistantContent } : m));
            return [...prev, { id: assistantId, role: "assistant", content: assistantContent }];
          });
        },
        onDone: () => { setIsTyping(false); abortRef.current = null; void refreshSessionInfo(); },
        onError: (err) => {
          setMessages((prev) => [...prev, { id: crypto.randomUUID(), role: "assistant", content: `⚠️ ${err}` }]);
          setIsTyping(false);
          abortRef.current = null;
        },
      });
      return;
    }

    setIsTyping(true);
    setTimeout(() => {
      setMessages((prev) => [
        ...prev,
        {
          id: crypto.randomUUID(),
          role: "assistant",
          content: "No local model active. Open **Settings → Local Models** to download one, or connect a cloud provider.",
        },
      ]);
      setIsTyping(false);
    }, 250);
  }, [input, messages, handleCommand, refreshSessionInfo]);

  const quickActions = [
    { label: "New Note", icon: FileText, action: () => setInput("new note: ") },
    { label: "New Task", icon: Columns3, action: () => setInput("new task: ") },
  ];

  return (
    <div className={`flex flex-col h-full ${compact ? "" : "rounded-2xl card-3d-elevated"} overflow-hidden`}>
      {!compact && (
        <div className="flex items-center gap-2 px-4 py-3 border-b border-border">
          <div className="h-7 w-7 rounded-full bg-foreground/10 flex items-center justify-center">
            <Bot className="h-4 w-4 text-foreground" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="text-sm font-semibold text-foreground">Assistant</div>
            {sessionInfo ? (
              <div className="flex items-center gap-1 text-[9px] font-mono text-muted-foreground">
                <span className="text-primary/80">{sessionInfo.modelAlias}</span>
                <span className="text-muted-foreground/40">·</span>
                <span>{fmtK(sessionInfo.usedTokens)}/{fmtK(sessionInfo.nCtx)} ctx</span>
                <span className="text-muted-foreground/40">·</span>
                <span>{sessionInfo.temperature.toFixed(1)}°</span>
              </div>
            ) : (
              <div className="text-[10px] text-muted-foreground">Always ready to help</div>
            )}
          </div>
          <ModelSwitcher
            localModels={localModels}
            downloadedIds={downloadedIds}
            activeLocalId={activeLocalId}
            onLocalSelect={handleLocalSelect}
          />
        </div>
      )}

      {/* Messages */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto p-3 space-y-3 min-h-0">
        {messages.map((msg) => (
          <div key={msg.id} className={`flex gap-2 ${msg.role === "user" ? "justify-end" : "justify-start"}`}>
            {msg.role === "assistant" && (
              <div className="h-6 w-6 rounded-full bg-foreground/10 flex items-center justify-center shrink-0 mt-0.5">
                <Bot className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
            )}
            <div className={`max-w-[80%] rounded-2xl px-3 py-2 text-sm leading-relaxed ${
              msg.role === "user" ? "bg-foreground text-background rounded-br-md" : "bg-accent text-accent-foreground rounded-bl-md"
            }`}>
              {msg.content.split(/(\*\*[^*]+\*\*)/).map((part, i) =>
                part.startsWith("**") && part.endsWith("**") ? (
                  <strong key={i} className="font-semibold">{part.slice(2, -2)}</strong>
                ) : (
                  <span key={i}>{part}</span>
                )
              )}
            </div>
            {msg.role === "user" && (
              <div className="h-6 w-6 rounded-full bg-foreground/20 flex items-center justify-center shrink-0 mt-0.5">
                <User className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
            )}
          </div>
        ))}
        {isTyping && !messages.some((m) => m.role === "assistant" && m.content === "") && (
          <div className="flex gap-2">
            <div className="h-6 w-6 rounded-full bg-foreground/10 flex items-center justify-center shrink-0">
              <Bot className="h-3.5 w-3.5 text-muted-foreground" />
            </div>
            <div className="bg-accent rounded-2xl rounded-bl-md px-4 py-2.5">
              <div className="flex gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground animate-bounce" style={{ animationDelay: "0ms" }} />
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground animate-bounce" style={{ animationDelay: "150ms" }} />
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground animate-bounce" style={{ animationDelay: "300ms" }} />
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Quick Actions + Model Switcher */}
      <div className="flex gap-1.5 px-3 pb-1.5 items-center">
        {quickActions.map((qa) => (
          <button
            key={qa.label}
            onClick={() => { qa.action(); inputRef.current?.focus(); }}
            className="flex items-center gap-1 text-[10px] font-medium text-muted-foreground bg-muted hover:bg-accent hover:text-foreground rounded-full px-2.5 py-1 transition-colors"
          >
            <qa.icon className="h-3 w-3" />
            {qa.label}
          </button>
        ))}
        {compact && (
          <div className="ml-auto">
            <ModelSwitcher
              localModels={localModels}
              downloadedIds={downloadedIds}
              activeLocalId={activeLocalId}
              onLocalSelect={handleLocalSelect}
            />
          </div>
        )}
      </div>

      {/* Input */}
      <div className="p-2 border-t border-border">
        <div className="flex items-center gap-2 bg-muted rounded-xl px-3 py-2">
          <input
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSend()}
            placeholder={lfmActive ? `Ask ${providerLabel}...` : "Type a message..."}
            className="flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground outline-none"
          />
          <button
            onClick={handleSend}
            disabled={!input.trim()}
            className="h-7 w-7 rounded-lg bg-foreground text-background flex items-center justify-center hover:opacity-80 transition-opacity disabled:opacity-30"
          >
            <Send className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
}
