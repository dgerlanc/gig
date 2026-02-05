# Multi-language .gitignore Generation

## Overview

Add support for combining multiple language/tool templates into a single `.gitignore` file without duplicated patterns.

## CLI Interface

```bash
gig <languages> [output]
```

Languages are positional, comma-separated. The `-l/--lang` flag is removed (breaking change).

### Examples

```bash
gig python                    # single language
gig go,godot,emacs            # multiple languages
gig go,godot,emacs .gitignore # explicit output path
gig --list                    # list available templates (unchanged)
gig --help                    # show help (unchanged)
gig --version                 # show version (unchanged)
```

## Merge Logic

1. Parse comma-separated languages from first positional argument
2. Resolve each language using existing `get_template()` (exact match, then prefix match)
3. If any language fails to resolve, error immediately and write nothing
4. For each template in user-specified order:
   - Read all lines
   - For each line:
     - If comment (starts with `#`) or blank: always include
     - If pattern: include only if not seen before (exact match)
   - Track seen patterns in a `HashSet<&str>`
5. Write combined output

### Deduplication Rules

- Exact string match only (`*.log` and `*.LOG` are distinct)
- First occurrence wins; subsequent duplicates are silently dropped
- Comments and blank lines are never deduplicated

### Output Format

Templates are concatenated in user-specified order. Original comments are preserved. No added section headers.

```gitignore
# Binaries for programs and plugins
*.exe
*.dll

# Godot-specific ignores
.godot/

# Org-mode
*~
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| No arguments | Show help (unchanged) |
| Invalid language in list | Error: `no template found for language "X"` + exit 1 |
| Ambiguous language in list | Error: `ambiguous language "X"; matches: a, b` + exit 1 |
| Empty language (e.g., `go,,godot`) | Error: `empty language in list` + exit 1 |
| Output file exists | Error: `file .gitignore already exists` + exit 1 (unchanged) |

## Breaking Changes

- The `-l/--lang` flag is removed
- Languages are now positional arguments
- This is acceptable as the tool is pre-1.0

## Non-Goals

- Semantic merging by category (templates don't use consistent section names)
- Case-insensitive deduplication (filesystem behavior varies)
- Partial success (writing some templates if others fail)
