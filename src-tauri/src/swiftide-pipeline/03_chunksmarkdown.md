# 03 Chunks Markdown

## Goal
- Make Markdown the first-class chunking strategy for notes.

## Philosophy
- Respect author intent encoded in headings, lists, quotes, and fences.
- Preserve note readability in retrieved chunks.
- Keep heading ancestry attached to every chunk.

## Markdown Split Rules
- Split primarily by heading hierarchy:
- `#` top sections
- `##` subtopics
- `###` and deeper only when size requires
- Keep short child sections merged with parent when isolated content is too thin

## List Handling
- Preserve bullet/numbered list blocks together when semantically related
- Avoid splitting checklist items away from their heading unless size requires
- Preserve indentation and nesting

## Quote / Callout Handling
- Keep quoted or callout blocks intact
- Attach nearest heading path

## Table Handling
- Prefer table as one chunk if small
- For large tables, store summary chunk + row-group chunks
- Preserve header row context

## Wiki Links / Tags
- Extract and store separately
- Keep raw inline syntax in chunk text
- Enrich metadata with parsed links/tags for retrieval

## Context Preservation
- Prepend compact heading breadcrumb when embedding/retrieving
- Example:
```text
Note Title > Project X > Risks
```

## Dependencies Needed
- markdown parser / AST
- frontmatter parser
- wiki link parser
- tag extractor

## Core Interfaces
- `MarkdownSection`
- `HeadingBreadcrumb`
- `MarkdownChunker`
- `MarkdownChunkPolicy`

## Pseudocode
```text
parse markdown
build heading tree
for each node:
  collect descendant prose/lists/tables until token limit
  preserve breadcrumb
  emit markdown-aware chunk
```

## Potential Pitfalls
- Splitting headings from their content
- Flattening nested lists into low-value text
- Losing table semantics
- Breaking wiki links during normalization
