import { BookOpen, Columns3, LoaderCircle, Network, Plus, Moon, Sun } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { useStore } from "@/lib/store";
import { Button } from "@/components/ui/button";
import { useState, useEffect } from "react";
import type { ViewMode } from "@/lib/types";

const views: { id: ViewMode; title: string; icon: typeof BookOpen }[] = [
  { id: "notebook", title: "Notebook", icon: BookOpen },
  { id: "kanban", title: "Kanban Board", icon: Columns3 },
  { id: "graph", title: "Knowledge Graph", icon: Network },
];

export function AppSidebar() {
  const { state } = useSidebar();
  const collapsed = state === "collapsed";
  const { activeView, setActiveView, addNote, indexingStatus } = useStore();
  const [dark, setDark] = useState(() =>
    document.documentElement.classList.contains("dark")
  );
  const isIndexing = indexingStatus.progress < 100;

  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
  }, [dark]);

  return (
    <Sidebar collapsible="icon" className="border-r border-border bg-sidebar-background">
      <SidebarContent className="flex flex-col h-full">
        <SidebarGroup>
          {!collapsed && (
            <SidebarGroupLabel className="text-xs font-bold uppercase tracking-widest text-muted-foreground/70 mb-1">
              Zettelkasten
            </SidebarGroupLabel>
          )}
          {!collapsed && isIndexing && (
            <div className="mb-2 rounded-lg border border-border/70 bg-muted/30 px-3 py-2">
              <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
                <LoaderCircle className="h-3.5 w-3.5 animate-spin" />
                <span>A indexar notas...</span>
                <span className="ml-auto font-mono">{Math.round(indexingStatus.progress)}%</span>
              </div>
              <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-border/70">
                <div
                  className="h-full rounded-full bg-foreground transition-all duration-300"
                  style={{ width: `${indexingStatus.progress}%` }}
                />
              </div>
            </div>
          )}
          <SidebarGroupContent>
            <SidebarMenu>
              {views.map((v) => (
                <SidebarMenuItem key={v.id}>
                  <SidebarMenuButton
                    onClick={() => setActiveView(v.id)}
                    className={`cursor-pointer transition-colors ${
                      activeView === v.id
                        ? "bg-accent text-accent-foreground font-semibold"
                        : "text-muted-foreground hover:bg-accent/50"
                    }`}
                  >
                    <v.icon className="h-4 w-4 shrink-0" />
                    {!collapsed && <span>{v.title}</span>}
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <div className="mt-auto p-3 space-y-2">
          <Button
            size={collapsed ? "icon" : "default"}
            className="w-full"
            onClick={() => addNote()}
          >
            <Plus className="h-4 w-4" />
            {!collapsed && <span>New Note</span>}
          </Button>
          <Button
            variant="ghost"
            size={collapsed ? "icon" : "default"}
            className="w-full"
            onClick={() => setDark(!dark)}
          >
            {dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
            {!collapsed && <span>{dark ? "Light" : "Dark"}</span>}
          </Button>
        </div>
      </SidebarContent>
    </Sidebar>
  );
}
