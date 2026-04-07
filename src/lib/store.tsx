import React, { createContext, useContext, useState, useCallback, useEffect, useRef } from "react";
import { Note, KanbanColumn, DEFAULT_COLUMNS, ViewMode } from "./types";
import { loadAgentNotes, saveAgentNotes } from "./crypto";
import {
  createWorkspaceFolder,
  deleteWorkspaceNote,
  isTauriRuntimeAvailable,
  loadWorkspaceSnapshot,
  saveWorkspaceNote,
} from "./commands";

const FOLDERS_KEY = "vibo-folders";

function loadFolders(): string[] {
  try {
    return JSON.parse(localStorage.getItem(FOLDERS_KEY) || "[]");
  } catch { return []; }
}
function saveFolders(folders: string[]) {
  localStorage.setItem(FOLDERS_KEY, JSON.stringify(folders));
}

interface StoreContextType {
  notes: Note[];
  agentNotes: Note[];
  columns: KanbanColumn[];
  folders: string[];
  activeView: ViewMode;
  selectedNoteId: string | null;
  isHydrating: boolean;
  setActiveView: (v: ViewMode) => void;
  selectNote: (id: string | null) => void;
  addNote: (column?: string, isKanban?: boolean, seed?: Partial<Note>) => Note;
  updateNote: (id: string, updates: Partial<Note>) => void;
  deleteNote: (id: string) => void;
  moveNote: (id: string, column: string, position: number) => void;
  addAgentNote: (note: Partial<Note>) => Note;
  getAgentNotes: () => Note[];
  addFolder: (name: string) => void;
  moveNoteToFolder: (noteId: string, folder: string) => void;
  toggleNoteEncryption: (noteId: string) => void;
  getNoteSaveState: (noteId: string) => "idle" | "saving" | "saved" | "error";
}

const StoreContext = createContext<StoreContextType | null>(null);

const COLUMNS_KEY = "zettel-columns";

function loadColumns(): KanbanColumn[] {
  try {
    const raw = localStorage.getItem(COLUMNS_KEY);
    return raw ? JSON.parse(raw) : DEFAULT_COLUMNS;
  } catch {
    return DEFAULT_COLUMNS;
  }
}

interface StoreProviderProps {
  children: React.ReactNode;
  pin: string;
  initialNotes: Note[];
}

export function StoreProvider({ children, pin, initialNotes }: StoreProviderProps) {
  const [notes, setNotes] = useState<Note[]>(initialNotes);
  const [agentNotes, setAgentNotes] = useState<Note[]>(() => {
    try { return JSON.parse(loadAgentNotes()); } catch { return []; }
  });
  const [columns] = useState<KanbanColumn[]>(loadColumns);
  const [folders, setFolders] = useState<string[]>(loadFolders);
  const [activeView, setActiveView] = useState<ViewMode>("dashboard");
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [isHydrating, setIsHydrating] = useState(false);
  const [noteSaveState, setNoteSaveState] = useState<Record<string, "idle" | "saving" | "saved" | "error">>({});
  const persistTimersRef = useRef<Map<string, number>>(new Map());

  useEffect(() => {
    return () => {
      persistTimersRef.current.forEach((timer) => window.clearTimeout(timer));
      persistTimersRef.current.clear();
    };
  }, []);

  useEffect(() => {
    if (!isTauriRuntimeAvailable()) {
      return;
    }

    let cancelled = false;
    setIsHydrating(true);

    const hydrate = async () => {
      try {
        const snapshot = await loadWorkspaceSnapshot();
        if (cancelled) return;

        const privateNotes = initialNotes.filter((note) => note.isEncrypted);
        const legacyWorkspaceNotes = initialNotes.filter((note) => !note.isEncrypted);

        const backendIds = new Set(snapshot.notes.map((note) => note.id));
        const missingLegacyNotes = legacyWorkspaceNotes.filter((note) => !backendIds.has(note.id));

        for (const note of missingLegacyNotes) {
          await saveWorkspaceNote(note);
        }

        const mergedNotes = mergeNotes(snapshot.notes, missingLegacyNotes, privateNotes);
        const mergedFolders = Array.from(new Set([...snapshot.folders, ...folders])).sort((a, b) => a.localeCompare(b));

        for (const folder of mergedFolders) {
          await createWorkspaceFolder(folder);
        }

        if (!cancelled) {
          setNotes(mergedNotes);
          setFolders(mergedFolders);
        }
      } catch (error) {
        console.error("Failed to hydrate workspace from Tauri backend:", error);
      } finally {
        if (!cancelled) {
          setIsHydrating(false);
        }
      }
    };

    hydrate();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    saveAgentNotes(JSON.stringify(agentNotes));
  }, [agentNotes]);

  useEffect(() => {
    saveFolders(folders);
  }, [folders]);

  const persistNote = useCallback((note: Note, debounceMs = 0) => {
    if (!isTauriRuntimeAvailable()) {
      return;
    }

    const existingTimer = persistTimersRef.current.get(note.id);
    if (existingTimer) {
      window.clearTimeout(existingTimer);
      persistTimersRef.current.delete(note.id);
    }

    setNoteSaveState((prev) => ({ ...prev, [note.id]: "saving" }));

    const persist = () => {
      void saveWorkspaceNote(note)
        .then(() => {
          setNoteSaveState((prev) => ({ ...prev, [note.id]: "saved" }));
        })
        .catch((error) => {
          console.error(`Failed to persist note ${note.id}:`, error);
          setNoteSaveState((prev) => ({ ...prev, [note.id]: "error" }));
          if (String(error).includes("Vault locked")) {
            window.dispatchEvent(new CustomEvent("vibo:vault-locked"));
          }
        });
    };

    if (debounceMs <= 0) {
      persist();
      return;
    }

    const timer = window.setTimeout(() => {
      persistTimersRef.current.delete(note.id);
      persist();
    }, debounceMs);

    persistTimersRef.current.set(note.id, timer);
  }, []);

  const addNote = useCallback((column = "inbox", isKanban = false, seed: Partial<Note> = {}) => {
    const now = new Date().toISOString();
    const kanbanTemplate = isKanban
      ? "## Task\n\n**Status:** To Do\n**Priority:** Medium\n\n### Description\n\n\n### Acceptance Criteria\n- [ ] \n"
      : "";
    const note: Note = {
      id: seed.id || crypto.randomUUID(),
      title: seed.title || "Untitled",
      content: seed.content ?? kanbanTemplate,
      tags: seed.tags || [],
      column: seed.column || column,
      position: seed.position ?? Date.now(),
      isKanban: seed.isKanban ?? isKanban,
      createdAt: seed.createdAt || now,
      updatedAt: seed.updatedAt || now,
      folder: seed.folder,
      isEncrypted: seed.isEncrypted,
    };
    setNotes((prev) => [note, ...prev]);
    setSelectedNoteId(note.id);
    persistNote(note);
    return note;
  }, [persistNote]);

  const updateNote = useCallback((id: string, updates: Partial<Note>) => {
    let nextNote: Note | null = null;
    setNotes((prev) =>
      prev.map((n) => {
        if (n.id !== id) return n;
        nextNote = { ...n, ...updates, updatedAt: new Date().toISOString() };
        return nextNote;
      })
    );

    if (nextNote) {
      persistNote(nextNote, 350);
    }
  }, [persistNote]);

  const deleteNote = useCallback((id: string) => {
    const note = notes.find((n) => n.id === id);
    const existingTimer = persistTimersRef.current.get(id);
    if (existingTimer) {
      window.clearTimeout(existingTimer);
      persistTimersRef.current.delete(id);
    }
    setNotes((prev) => prev.filter((n) => n.id !== id));
    setSelectedNoteId((prev) => (prev === id ? null : prev));
    if (note && isTauriRuntimeAvailable()) {
      setNoteSaveState((prev) => ({ ...prev, [id]: "saving" }));
      void deleteWorkspaceNote(id).catch((error) => {
        console.error(`Failed to delete note ${id}:`, error);
        setNoteSaveState((prev) => ({ ...prev, [id]: "error" }));
        if (String(error).includes("Vault locked")) {
          window.dispatchEvent(new CustomEvent("vibo:vault-locked"));
        }
      });
    }
  }, [notes]);

  const moveNote = useCallback((id: string, column: string, position: number) => {
    let nextNote: Note | null = null;
    setNotes((prev) =>
      prev.map((n) => {
        if (n.id !== id) return n;
        nextNote = { ...n, column, position, updatedAt: new Date().toISOString() };
        return nextNote;
      })
    );
    if (nextNote) {
      persistNote(nextNote);
    }
  }, [persistNote]);

  const selectNote = useCallback((id: string | null) => {
    setSelectedNoteId(id);
  }, []);

  const addAgentNote = useCallback((partial: Partial<Note>) => {
    const now = new Date().toISOString();
    const note: Note = {
      id: crypto.randomUUID(),
      title: partial.title || "Agent Note",
      content: partial.content || "",
      tags: partial.tags || ["agent"],
      column: "agent",
      position: Date.now(),
      isKanban: false,
      createdAt: now,
      updatedAt: now,
    };
    setAgentNotes((prev) => [note, ...prev]);
    return note;
  }, []);

  const getAgentNotes = useCallback(() => agentNotes, [agentNotes]);

  const addFolder = useCallback((name: string) => {
    setFolders((prev) => {
      if (prev.includes(name)) return prev;
      if (isTauriRuntimeAvailable()) {
        void createWorkspaceFolder(name).catch((error) => {
          console.error(`Failed to create folder ${name}:`, error);
        });
      }
      return [...prev, name];
    });
  }, []);

  const moveNoteToFolder = useCallback((noteId: string, folder: string) => {
    let nextNote: Note | null = null;
    setNotes((prev) =>
      prev.map((n) => {
        if (n.id !== noteId) return n;
        nextNote = { ...n, folder, updatedAt: new Date().toISOString() };
        return nextNote;
      })
    );
    if (nextNote) {
      persistNote(nextNote);
    }
  }, [persistNote]);

  const toggleNoteEncryption = useCallback((noteId: string) => {
    let nextNote: Note | null = null;
    setNotes((prev) =>
      prev.map((n) => {
        if (n.id !== noteId) return n;
        nextNote = {
          ...n,
          isEncrypted: !n.isEncrypted,
          updatedAt: new Date().toISOString(),
        };
        return nextNote;
      })
    );

    if (!nextNote || !isTauriRuntimeAvailable()) {
      return;
    }

    const existingTimer = persistTimersRef.current.get(noteId);
    if (existingTimer) {
      window.clearTimeout(existingTimer);
      persistTimersRef.current.delete(noteId);
    }

    persistNote(nextNote);
  }, [persistNote]);

  const getNoteSaveState = useCallback((noteId: string) => {
    if (noteSaveState[noteId]) {
      return noteSaveState[noteId];
    }
    return "idle";
  }, [noteSaveState]);

  return (
    <StoreContext.Provider
      value={{
        notes,
        agentNotes,
        columns,
        folders,
        activeView,
        selectedNoteId,
        isHydrating,
        setActiveView,
        selectNote,
        addNote,
        updateNote,
        deleteNote,
        moveNote,
        addAgentNote,
        getAgentNotes,
        addFolder,
        moveNoteToFolder,
        toggleNoteEncryption,
        getNoteSaveState,
      }}
    >
      {children}
    </StoreContext.Provider>
  );
}

export function useStore() {
  const ctx = useContext(StoreContext);
  if (!ctx) throw new Error("useStore must be inside StoreProvider");
  return ctx;
}

function mergeNotes(...groups: Note[][]): Note[] {
  const byId = new Map<string, Note>();

  for (const group of groups) {
    for (const note of group) {
      byId.set(note.id, note);
    }
  }

  return Array.from(byId.values()).sort((a, b) => {
    const left = Date.parse(b.updatedAt || b.createdAt || "");
    const right = Date.parse(a.updatedAt || a.createdAt || "");
    return left - right;
  });
}
