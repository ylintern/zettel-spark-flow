import { useMemo, useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { Note } from "@/lib/types";

interface SprintCalendarProps {
  tasks: Note[];
  onTaskClick?: (noteId: string) => void;
}

type CalendarView = "week" | "month" | "quarter";

function getDaysInMonth(year: number, month: number) {
  return new Date(year, month + 1, 0).getDate();
}

function getFirstDayOfWeek(year: number, month: number) {
  return new Date(year, month, 1).getDay();
}

function getWeekStart(date: Date) {
  const d = new Date(date);
  d.setDate(d.getDate() - d.getDay());
  d.setHours(0, 0, 0, 0);
  return d;
}

const WEEKDAYS = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];
const MONTHS_SHORT = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

function dateKey(y: number, m: number, d: number) {
  return `${y}-${String(m + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
}

function getColumnColor(col: string) {
  switch (col) {
    case "done": return "bg-primary";
    case "in-progress": return "bg-accent-foreground/60";
    case "review": return "bg-muted-foreground/60";
    default: return "bg-muted-foreground/30";
  }
}

function TaskDots({ tasks, onTaskClick }: { tasks: Note[]; onTaskClick?: (id: string) => void }) {
  if (tasks.length === 0) return null;
  return (
    <div className="flex gap-0.5 mt-0.5">
      {tasks.slice(0, 3).map((t) => (
        <div key={t.id} className={`w-1.5 h-1.5 rounded-full ${getColumnColor(t.column)}`} title={t.title} />
      ))}
      {tasks.length > 3 && (
        <span className="text-[7px] text-muted-foreground leading-none">+{tasks.length - 3}</span>
      )}
    </div>
  );
}

export function SprintCalendar({ tasks, onTaskClick }: SprintCalendarProps) {
  const today = new Date();
  const [view, setView] = useState<CalendarView>("month");
  const [viewMonth, setViewMonth] = useState(today.getMonth());
  const [viewYear, setViewYear] = useState(today.getFullYear());
  const [weekStart, setWeekStart] = useState(() => getWeekStart(today));

  const tasksByDate = useMemo(() => {
    const map: Record<string, Note[]> = {};
    for (const t of tasks) {
      const dk = t.updatedAt.slice(0, 10);
      if (!map[dk]) map[dk] = [];
      map[dk].push(t);
    }
    return map;
  }, [tasks]);

  const todayStr = dateKey(today.getFullYear(), today.getMonth(), today.getDate());

  // Navigation
  const prev = () => {
    if (view === "week") {
      setWeekStart((w) => { const d = new Date(w); d.setDate(d.getDate() - 7); return d; });
    } else if (view === "month") {
      if (viewMonth === 0) { setViewMonth(11); setViewYear((y) => y - 1); }
      else setViewMonth((m) => m - 1);
    } else {
      setViewMonth((m) => {
        const nm = m - 3;
        if (nm < 0) { setViewYear((y) => y - 1); return nm + 12; }
        return nm;
      });
    }
  };
  const next = () => {
    if (view === "week") {
      setWeekStart((w) => { const d = new Date(w); d.setDate(d.getDate() + 7); return d; });
    } else if (view === "month") {
      if (viewMonth === 11) { setViewMonth(0); setViewYear((y) => y + 1); }
      else setViewMonth((m) => m + 1);
    } else {
      setViewMonth((m) => {
        const nm = m + 3;
        if (nm > 11) { setViewYear((y) => y + 1); return nm - 12; }
        return nm;
      });
    }
  };

  const headerLabel = () => {
    if (view === "week") {
      const end = new Date(weekStart);
      end.setDate(end.getDate() + 6);
      return `${MONTHS_SHORT[weekStart.getMonth()]} ${weekStart.getDate()} – ${MONTHS_SHORT[end.getMonth()]} ${end.getDate()}, ${end.getFullYear()}`;
    }
    if (view === "quarter") {
      const q = Math.floor(viewMonth / 3) + 1;
      return `Q${q} ${viewYear}`;
    }
    return `${MONTHS[viewMonth]} ${viewYear}`;
  };

  // Render day cell
  const renderDay = (date: Date, compact = false) => {
    const dk = dateKey(date.getFullYear(), date.getMonth(), date.getDate());
    const isToday = dk === todayStr;
    const dayTasks = tasksByDate[dk] || [];

    return (
      <div
        key={dk}
        className={`relative flex flex-col items-center py-1 rounded-lg transition-colors ${
          isToday ? "bg-accent" : ""
        } ${dayTasks.length > 0 ? "cursor-pointer hover:bg-accent/60" : ""}`}
        onClick={() => { if (dayTasks.length > 0 && onTaskClick) onTaskClick(dayTasks[0].id); }}
      >
        <span className={`text-[11px] leading-none ${isToday ? "font-bold text-foreground" : "text-muted-foreground"}`}>
          {compact ? date.getDate() : date.getDate()}
        </span>
        <TaskDots tasks={dayTasks} onTaskClick={onTaskClick} />
      </div>
    );
  };

  // Week view
  const renderWeek = () => {
    const days: Date[] = [];
    for (let i = 0; i < 7; i++) {
      const d = new Date(weekStart);
      d.setDate(d.getDate() + i);
      days.push(d);
    }
    return (
      <>
        <div className="grid grid-cols-7 px-2">
          {WEEKDAYS.map((d) => (
            <div key={d} className="text-center text-[9px] font-mono text-muted-foreground py-1 uppercase tracking-wider">{d}</div>
          ))}
        </div>
        <div className="grid grid-cols-7 px-2 flex-1">
          {days.map((d) => renderDay(d))}
        </div>
      </>
    );
  };

  // Month view
  const renderMonth = () => {
    const daysInMonth = getDaysInMonth(viewYear, viewMonth);
    const firstDay = getFirstDayOfWeek(viewYear, viewMonth);
    const cells: (Date | null)[] = [];
    for (let i = 0; i < firstDay; i++) cells.push(null);
    for (let d = 1; d <= daysInMonth; d++) cells.push(new Date(viewYear, viewMonth, d));

    return (
      <>
        <div className="grid grid-cols-7 px-2">
          {WEEKDAYS.map((d) => (
            <div key={d} className="text-center text-[9px] font-mono text-muted-foreground py-1 uppercase tracking-wider">{d}</div>
          ))}
        </div>
        <div className="grid grid-cols-7 px-2 gap-y-0.5 flex-1">
          {cells.map((d, i) => d ? renderDay(d) : <div key={`e-${i}`} />)}
        </div>
      </>
    );
  };

  // Quarter view — 3 mini months
  const renderQuarter = () => {
    const qStart = Math.floor(viewMonth / 3) * 3;
    return (
      <div className="grid grid-cols-3 gap-1 px-2 flex-1 overflow-hidden">
        {[0, 1, 2].map((offset) => {
          const m = qStart + offset;
          const daysInM = getDaysInMonth(viewYear, m);
          const firstD = getFirstDayOfWeek(viewYear, m);
          const cells: (Date | null)[] = [];
          for (let i = 0; i < firstD; i++) cells.push(null);
          for (let d = 1; d <= daysInM; d++) cells.push(new Date(viewYear, m, d));

          return (
            <div key={m} className="flex flex-col min-w-0">
              <div className="text-[8px] font-mono text-muted-foreground uppercase tracking-widest text-center py-0.5">{MONTHS_SHORT[m]}</div>
              <div className="grid grid-cols-7 gap-0">
                {cells.map((d, i) => {
                  if (!d) return <div key={`e-${m}-${i}`} />;
                  const dk = dateKey(d.getFullYear(), d.getMonth(), d.getDate());
                  const isToday = dk === todayStr;
                  const dayTasks = tasksByDate[dk] || [];
                  return (
                    <div
                      key={dk}
                      className={`flex items-center justify-center h-4 rounded-sm ${isToday ? "bg-accent" : ""} ${dayTasks.length > 0 ? "cursor-pointer" : ""}`}
                      onClick={() => { if (dayTasks.length > 0 && onTaskClick) onTaskClick(dayTasks[0].id); }}
                    >
                      <span className={`text-[7px] leading-none ${isToday ? "font-bold text-foreground" : dayTasks.length > 0 ? "text-primary font-bold" : "text-muted-foreground/60"}`}>
                        {d.getDate()}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2">
        <button onClick={prev} className="p-1 rounded-md hover:bg-accent transition-colors text-muted-foreground hover:text-foreground">
          <ChevronLeft className="h-4 w-4" />
        </button>
        <div className="flex flex-col items-center gap-1">
          <span className="text-xs font-bold text-foreground tracking-tight font-mono uppercase">
            {headerLabel()}
          </span>
          {/* View switcher */}
          <div className="flex gap-0.5 bg-muted rounded-full p-0.5">
            {(["week", "month", "quarter"] as CalendarView[]).map((v) => (
              <button
                key={v}
                onClick={() => setView(v)}
                className={`text-[8px] font-mono uppercase tracking-wider px-2 py-0.5 rounded-full transition-colors ${
                  view === v ? "bg-foreground text-background" : "text-muted-foreground hover:text-foreground"
                }`}
              >
                {v === "quarter" ? "Q" : v.charAt(0).toUpperCase()}
              </button>
            ))}
          </div>
        </div>
        <button onClick={next} className="p-1 rounded-md hover:bg-accent transition-colors text-muted-foreground hover:text-foreground">
          <ChevronRight className="h-4 w-4" />
        </button>
      </div>

      {/* Content */}
      {view === "week" && renderWeek()}
      {view === "month" && renderMonth()}
      {view === "quarter" && renderQuarter()}
    </div>
  );
}
