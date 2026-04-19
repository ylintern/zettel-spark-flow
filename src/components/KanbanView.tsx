import { useState, useMemo, useRef } from "react";
import { useStore } from "@/lib/store";
import { NoteEditor } from "./NoteEditor";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Plus, GripVertical } from "lucide-react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Sheet, SheetContent, SheetTitle, SheetDescription } from "@/components/ui/sheet";

export function KanbanView() {
  const { notes, columns, addNote, moveNote } = useStore();
  const [sheetNoteId, setSheetNoteId] = useState<string | null>(null);
  const dragRef = useRef<{ noteId: string; sourceColumn: string } | null>(null);

  const kanbanNotes = useMemo(() => notes.filter((n) => n.isKanban), [notes]);

  const notesByColumn = useMemo(() => {
    const map: Record<string, typeof notes> = {};
    columns.forEach((c) => {
      map[c.id] = kanbanNotes
        .filter((n) => n.status === c.id)
        .sort((a, b) => a.position - b.position);
    });
    return map;
  }, [kanbanNotes, columns]);

  const handleDragStart = (e: React.DragEvent, noteId: string, column: string) => {
    console.log("[Kanban] DragStart: " + noteId + " from " + column);
    dragRef.current = { noteId, sourceColumn: column };
    e.dataTransfer.setData("text/plain", noteId);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("application/x-note-id", noteId);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = "move";
  };

  const handleDragEnd = (_e: React.DragEvent) => {
    // Do not clear dragRef here — in WKWebView, dragend can fire before drop
    // on the target. handleDrop clears it on success.
  };

  const handleDrop = (e: React.DragEvent, columnId: string) => {
    e.preventDefault();
    e.stopPropagation();

    const noteId =
      dragRef.current?.noteId ?? e.dataTransfer.getData("application/x-note-id") ?? null;
    const sourceColumn = dragRef.current?.sourceColumn ?? null;

    if (noteId) {
      console.log("[Kanban] Moving " + noteId + " from " + (sourceColumn ?? "?") + " to " + columnId);
      moveNote(noteId, columnId, Date.now());
      dragRef.current = null;
    } else {
      console.warn("[Kanban] Drop failed - no noteId in dragRef or dataTransfer");
    }
  };

  return (
    <>
      <div className="flex h-full overflow-x-auto gap-2 p-3">
        {columns.map((col) => (
          <div
            key={col.id}
            className="flex flex-col w-56 shrink-0 rounded-xl bg-muted/30 border border-border overflow-hidden"
          >
            {/* Column header */}
            <div className="flex items-center justify-between px-2.5 py-2 border-b border-border bg-muted/30">
              <h3 className="font-semibold text-xs text-foreground">{col.title}</h3>
              <Badge variant="secondary" className="text-[10px] h-4 px-1.5">
                {notesByColumn[col.id]?.length || 0}
              </Badge>
            </div>

            {/* Cards - plain scrollable div for reliable HTML5 drag-and-drop */}
            <div
              className="flex-1 p-1.5 overflow-y-auto overflow-x-hidden"
              onDragOver={handleDragOver}
              onDragEnter={handleDragOver}
              onDrop={(e) => handleDrop(e, col.id)}
            >
              <div className="space-y-1.5 min-h-[100px]">
                {notesByColumn[col.id]?.map((note) => (
                  <div
                    key={note.id}
                    draggable
                    onDragStart={(e) => handleDragStart(e, note.id, col.id)}
                    onDragEnd={handleDragEnd}
                    onClick={() => setSheetNoteId(note.id)}
                    className="group cursor-pointer rounded-lg border border-border bg-card px-2.5 py-2 shadow-ambient hover:shadow-elevated transition-shadow"
                  >
                    <div className="flex items-start gap-1.5 pointer-events-none">
                      <GripVertical className="h-3 w-3 text-muted-foreground/40 mt-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity cursor-grab" />
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-xs truncate text-foreground">
                          {note.title}
                        </div>
                        <div className="text-[10px] text-muted-foreground mt-0.5 line-clamp-2">
                          {note.content.slice(0, 60) || "Empty"}
                        </div>
                        {note.tags.length > 0 && (
                          <div className="flex gap-0.5 mt-1.5 flex-wrap">
                            {note.tags.slice(0, 2).map((t) => (
                              <span
                                key={t}
                                className="text-[9px] bg-accent text-accent-foreground px-1 py-0.5 rounded"
                              >
                                #{t}
                              </span>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
                {/* Empty state drop zone */}
                {notesByColumn[col.id]?.length === 0 && (
                  <div className="h-24 border-2 border-dashed border-border/50 rounded-lg flex items-center justify-center text-muted-foreground text-xs pointer-events-none">
                    Drop here
                  </div>
                )}
              </div>
            </div>

            {/* Add card */}
            <div className="p-1.5 border-t border-border">
              <Button
                variant="ghost"
                size="sm"
                className="w-full text-muted-foreground text-xs h-7"
                onClick={() => addNote(col.id, true, { folder: "tasks" })}
              >
                <Plus className="h-3 w-3 mr-1" /> Add
              </Button>
            </div>
          </div>
        ))}
      </div>

      {/* Note editor sheet */}
      <Sheet open={!!sheetNoteId} onOpenChange={(open) => !open && setSheetNoteId(null)}>
        <SheetContent className="w-full sm:w-[500px] sm:max-w-[500px] p-0">
          <SheetTitle className="sr-only">Edit Note</SheetTitle>
          <SheetDescription className="sr-only">Edit and manage your note content</SheetDescription>
          {sheetNoteId && (
            <NoteEditor noteId={sheetNoteId} onClose={() => setSheetNoteId(null)} />
          )}
        </SheetContent>
      </Sheet>
    </>
  );
}
