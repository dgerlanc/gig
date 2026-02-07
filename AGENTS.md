# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project Overview

`gig` is a Rust CLI tool that generates `.gitignore` files from GitHub's template collection. Templates are embedded directly into the binary at compile time using the `include_dir!` macro.

## Build and Test Commands

```bash
cargo build              # Build debug binary
cargo build --release    # Build release binary
cargo test               # Run all tests
cargo test <test_name>   # Run a single test (e.g., cargo test test_get_template_exact_match)
cargo run -- python      # Run with arguments
```

## Architecture

### Build step (`build.rs`)

A build script flattens all templates (including nested ones from `Global/` and `community/` subdirectories) into `$OUT_DIR/templates/` before compilation. Nested templates get dot-prefixed filenames to avoid collisions (e.g., `Global/macOS.gitignore` becomes `global.macOS.gitignore`).

### Runtime (`src/main.rs`)

- **Template index**: Built lazily using `LazyLock<HashMap>` - maps lowercase language names to template content
- **Template lookup**: Exact match only (case-insensitive). Nested templates use dot-notation (e.g., `global.macos`, `community.javascript.vue`)
- **Template merging**: Multiple templates are merged with pattern deduplication
- **Append mode**: `--append` reads the existing file, merges with new templates (deduplicating patterns), and overwrites
- **File writing**: Default uses `create_new(true)` to avoid overwriting; `--append` uses `create(true).truncate(true)` to replace

Key functions:
- `build_index()` - Builds the template HashMap from embedded files
- `get_template()` - Looks up templates with exact match (case-insensitive)
- `parse_args()` - Handles CLI argument parsing with pico-args
- `parse_languages()` - Parses comma-separated language list
- `merge_templates()` - Merges multiple templates with deduplication
- `write_output()` - Safe file creation (supports overwrite for append mode)
- `read_existing_file()` - Reads existing file content for append mode

## CLI Usage

```bash
gig <languages> [output]           # Generate .gitignore (output defaults to .gitignore)
gig python                         # Single language
gig go,godot,node                  # Multiple languages, comma-separated
gig python,global.macos            # Mix top-level and nested templates
gig rust,community.golang.hugo     # Community template with subcategory
gig --append node                  # Append Node patterns to existing .gitignore
gig --list                         # List available templates
gig --help                         # Show help
gig --version                      # Show version
```

Nested templates use dot-notation: `global.<name>` for Global/ templates, `community.<subcategory>.<name>` for community/ templates.

## Dependencies

- **pico-args**: Lightweight CLI argument parsing
- **include_dir**: Embeds template directory into binary at compile time
