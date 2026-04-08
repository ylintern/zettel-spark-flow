# 04 AST Tree-sitter Chunks

## Goal
- Chunk code inside notes by syntax boundaries instead of arbitrary line windows.

## Why AST Chunking Matters
- Code meaning follows functions, classes, modules, and blocks.
- Arbitrary splitting breaks compile-time or conceptual units.
- Retrieval quality improves when chunks map to valid code shapes.

## Target Use Cases
- fenced code blocks in Markdown notes
- language-specific snippets
- future code-note hybrid workspaces

## Strategy
- Detect fenced code blocks with language hints
- Route supported languages to Tree-sitter parser
- Build syntax-aware chunks:
- function
- method
- class
- struct
- module
- interface / trait
- fallback block group when parse is incomplete

## Chunk Rules
- Keep function signature with body
- Keep doc comments with the symbol they describe
- Keep imports/context when compact enough
- If symbol body is too large:
- split by nested statements only as last resort
- include parent symbol metadata

## Metadata
- `language`
- `symbol_name`
- `symbol_kind`
- `parent_symbol`
- `line_start`
- `line_end`
- `fence_info`

## Unsupported Languages
- Keep raw fenced block as a code chunk
- Mark parser status as fallback

## Dependencies Needed
- Tree-sitter core
- language grammars
- markdown code fence extraction
- source span tracking

## Core Interfaces
- `CodeFence`
- `AstChunker`
- `SymbolChunk`
- `ParserSupportRegistry`

## Pseudocode
```text
for each fenced code block:
  if grammar available:
    tree = parse(code)
    symbols = extract top-level semantic nodes
    emit symbol chunks
  else:
    emit fallback code chunk
```

## Potential Pitfalls
- Parse failures on incomplete snippets
- Heavy grammar footprint across platforms
- Over-segmentation of tiny functions
- Losing surrounding Markdown explanation if code is indexed alone
