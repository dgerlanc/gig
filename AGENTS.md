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

The entire application lives in `src/main.rs`:

- **Template index**: Built lazily using `LazyLock<HashMap>` - maps lowercase language names to template content
- **Template lookup**: Supports exact match (case-insensitive) with prefix matching fallback
- **File writing**: Uses `OpenOptions::create_new(true)` for atomic creation (won't overwrite existing files)

Key functions:
- `build_index()` - Builds the template HashMap from embedded files
- `get_template()` - Looks up templates with exact/prefix matching
- `parse_args()` - Handles CLI argument parsing with pico-args
- `write_output()` - Safe file creation

## CLI Usage

```bash
gig <languages> [output]      # Generate .gitignore (output defaults to .gitignore)
gig python                    # Single language
gig go,godot,node             # Multiple languages, comma-separated
gig --list                    # List available templates
gig --help                    # Show help
gig --version                 # Show version
```

## Dependencies

- **pico-args**: Lightweight CLI argument parsing
- **include_dir**: Embeds template directory into binary at compile time
