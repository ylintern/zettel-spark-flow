export const RESERVED_FOLDER_NAMES = [
  "notes",
  "tasks",
  "agents",
  "skills",
  "roles",
  "providers",
  "tools",
  "mcp",
  "plugin",
] as const;

export type ReservedFolderName = (typeof RESERVED_FOLDER_NAMES)[number];

export function isReservedFolder(name: string): boolean {
  return RESERVED_FOLDER_NAMES.includes(name.trim().toLowerCase() as ReservedFolderName);
}

export function defaultFolderFor(isKanban: boolean): "notes" | "tasks" {
  return isKanban ? "tasks" : "notes";
}
