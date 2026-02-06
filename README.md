# gig

A CLI tool that generates `.gitignore` files from [GitHub's gitignore template collection](https://github.com/github/gitignore). Templates are embedded directly in the binary, so it works offline after building.

## Quick Start

```sh
cargo build --release
./target/release/gig python
```

## Installation

### Homebrew (macOS/Linux)

```bash
brew install dgerlanc/tap/gig
```

### From GitHub Releases

Download the appropriate binary from the [releases page](https://github.com/dgerlanc/gig/releases).

### From Source

```sh
# 1. Clone this repo
git clone https://github.com/dgerlanc/gig.git
cd gig

# 2. Download templates
git clone https://github.com/github/gitignore.git templates

# 3. Build
cargo build --release

# 4. (Optional) Install to ~/.cargo/bin
cargo install --path .
```

## Usage

```sh
# Create .gitignore for Python in current directory
gig python

# Multiple languages, comma-separated
gig go,nextjs,terraform

# Specify an output path
gig rust src/.gitignore

# Use nested templates (Global/, community/)
gig global.macos
gig go,community.golang.hugo

# List all available languages
gig --list

# Show help
gig --help
```

Language matching is case-insensitive, so `gig Python` and `gig python` both work.

### Nested Templates

Templates from GitHub's `Global/` and `community/` subdirectories are available using dot-notation:

- **Global templates**: `global.<name>` (e.g., `global.macos`, `global.linux`)
- **Community templates**: `community.<subcategory>.<name>` (e.g., `community.javascript.vue`, `community.cfml.coldbox`)

Run `gig --list` to see all available template names.

## Updating Templates

To pull the latest templates from GitHub:

```sh
cd templates && git pull
cargo build --release
```

## Development

```sh
cargo build              # Build debug binary
cargo build --release    # Build release binary
cargo test               # Run all tests
cargo clippy             # Run linter
cargo fmt                # Format code
```

## How It Works

The `templates/` directory contains `.gitignore` files from `github/gitignore`. Rust's `include_dir!` macro bakes these files into the binary at compile time. The result is a single static binary with no runtime dependencies.

## License

APACHE 2.0
