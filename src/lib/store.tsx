import React, { createContext, useContext, useState, useCallback, useEffect, useRef } from "react";
import { Note, KanbanColumn, DEFAULT_COLUMNS, ViewMode } from "./types";
import { loadAgentNotes, saveAgentNotes } from "./crypto";
import {
  createWorkspaceFolder,
  deleteWorkspaceNote,
  isTauriRuntimeAvailable,
  loadWorkspaceSnapshot,
  onNoteIndexingProgress,
  type NoteIndexingProgressEvent,
  saveWorkspaceNote,
  saveWorkspaceColumn,
  fallbackPassphraseUnlock,
} from "./commands";

interface StoreContextType {
  notes: Note[];
  agentNotes: Note[];
  columns: KanbanColumn[];
  folders: string[];
  activeView: ViewMode;
  selectedNoteId: string | null;
  isHydrating: boolean;
  indexingStatus: NoteIndexingProgressEvent;
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

const DEFAULT_INDEXING_STATUS: NoteIndexingProgressEvent = {
  noteId: null,
  stage: "idle",
  progress: 100,
  processedNotes: 0,
  totalNotes: 0,
};

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
  const [columns, setColumns] = useState<KanbanColumn[]>([]);
  const [folders, setFolders] = useState<string[]>([]);
  const [activeView, setActiveView] = useState<ViewMode>("dashboard");
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [isHydrating, setIsHydrating] = useState(false);
  const [indexingStatus, setIndexingStatus] = useState<NoteIndexingProgressEvent>(DEFAULT_INDEXING_STATUS);
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

    let dispose = () => {};

    void onNoteIndexingProgress((payload) => {
      console.debug("[vibo] note_indexing_progress", payload);
      setIndexingStatus(payload);
    }).then((unlisten) => {
      dispose = () => {
        void unlisten();
      };
    });

    return () => {
      dispose();
    };
  }, []);

  useEffect(() => {
    if (!isTauriRuntimeAvailable()) {
      return;
    }

    let cancelled = false;
    setIsHydrating(true);
    setIndexingStatus({
      noteId: null,
      stage: "starting",
      progress: 0,
      processedNotes: 0,
      totalNotes: 0,
    });

    const hydrate = async () => {
      try {
        const snapshot = await loadWorkspaceSnapshot();
        if (cancelled) return;

        // All notes come from backend (Tauri hydration)
        const mergedNotes = snapshot.notes.sort((a, b) => {
          const left = Date.parse(b.updatedAt || b.createdAt || "");
          const right = Date.parse(a.updatedAt || a.createdAt || "");
          return left - right;
        });
        const mergedFolders = [...snapshot.folders].sort((a, b) => a.localeCompare(b));

        for (const folder of mergedFolders) {
          await createWorkspaceFolder(folder);
        }

        // Handle columns: seed DEFAULT_COLUMNS on first launch if backend is empty
        let columnsToUse = snapshot.columns;
        if (snapshot.columns.length === 0) {
          // First launch: seed with default columns
          columnsToUse = DEFAULT_COLUMNS;
          for (const column of DEFAULT_COLUMNS) {
            await saveWorkspaceColumn(column);
          }
        }

        if (!cancelled) {
          setNotes(mergedNotes);
          setFolders(mergedFolders);
          setColumns(columnsToUse);
        }
      } catch (error) {
        console.error("Failed to hydrate workspace from Tauri backend:", error);
      } finally {
        if (!cancelled) {
          setIsHydrating(false);
          setIndexingStatus((prev) => {
            if (prev.progress >= 100) {
              return prev;
            }

            return {
              ...prev,
              stage: "complete",
              progress: 100,
            };
          });
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
    if (!isTauriRuntimeAvailable()) return;

    const current = notes.find((n) => n.id === noteId);
    if (!current) return;

    const nextNote: Note = {
      ...current,
      isEncrypted: !current.isEncrypted,
      updatedAt: new Date().toISOString(),
    };

    setNotes((prev) => prev.map((n) => n.id === noteId ? nextNote : n));
    setNoteSaveState((prev) => ({ ...prev, [noteId]: "saving" }));

    void saveWorkspaceNote(nextNote)
      .then(() => {
        setNoteSaveState((prev) => ({ ...prev, [noteId]: "saved" }));
      })
      .catch((error) => {
        console.error(`Failed to toggle encryption for note ${noteId}:`, error);
        setNotes((prev) => prev.map((n) => n.id === noteId ? current : n));
        setNoteSaveState((prev) => ({ ...prev, [noteId]: "error" }));
        if (String(error).includes("Vault locked")) {
          window.dispatchEvent(new CustomEvent("vibo:vault-locked"));
        }
      });
  }, [notes, saveWorkspaceNote]);

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
        indexingStatus,
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
