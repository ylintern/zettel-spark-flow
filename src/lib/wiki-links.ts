import { Note } from "./types";

const WIKI_LINK_REGEX = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;

export interface WikiLink {
  target: string;
  display: string | null;
}

export function extractWikiLinks(content: string): WikiLink[] {
  const matches: WikiLink[] = [];
  let match: RegExpExecArray | null;
  while ((match = WIKI_LINK_REGEX.exec(content)) !== null) {
    matches.push({
      target: match[1],
      display: match[2] || null,
    });
  }
  return matches;
}

export function getBacklinks(noteTitle: string, allNotes: Note[]): Note[] {
  return allNotes.filter((n) => {
    const links = extractWikiLinks(n.content);
    return links.some((l) => l.target.toLowerCase() === noteTitle.toLowerCase());
  });
}

export function buildGraph(notes: Note[]): { nodes: GraphNode[]; edges: GraphEdge[] } {
  const nodes: GraphNode[] = notes.map((n) => ({
    id: n.id,
    title: n.title,
    linkCount: 0,
    isKanban: n.isKanban,
  }));

  const edges: GraphEdge[] = [];
  const titleToId = new Map(notes.map((n) => [n.title.toLowerCase(), n.id]));

  for (const note of notes) {
    const links = extractWikiLinks(note.content);
    for (const link of links) {
      const targetId = titleToId.get(link.toLowerCase());
      if (targetId && targetId !== note.id) {
        edges.push({ source: note.id, target: targetId });
        const sourceNode = nodes.find((n) => n.id === note.id);
        const targetNode = nodes.find((n) => n.id === targetId);
        if (sourceNode) sourceNode.linkCount++;
        if (targetNode) targetNode.linkCount++;
      }
    }
  }

  return { nodes, edges };
}

export interface GraphNode {
  id: string;
  title: string;
  linkCount: number;
  isKanban: boolean;
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
}

export interface GraphEdge {
  source: string;
  target: string;
}
