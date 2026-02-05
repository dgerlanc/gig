# Homebrew Release Design

## Overview

Automate building and releasing `gig` binaries via GitHub Actions, with distribution through a personal Homebrew tap.

## Goals

- Single-command release process
- Automated builds for macOS (Intel + Apple Silicon), Linux, and Windows
- Homebrew installation via personal tap

## Release Workflow

When a tag matching `v*` is pushed, GitHub Actions will:

1. Build binaries for all targets
2. Create a GitHub Release with artifacts
3. Update the Homebrew tap formula

### Build Targets

| Target | OS | Archive |
|--------|-----|---------|
| `x86_64-apple-darwin` | macOS Intel | `.tar.gz` |
| `aarch64-apple-darwin` | macOS Apple Silicon | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | Linux | `.tar.gz` |
| `x86_64-pc-windows-msvc` | Windows | `.zip` |

### Workflow Jobs

**Job 1: `build`**
- Matrix strategy across OS/target combinations
- Install Rust toolchain with appropriate target
- Run `cargo build --release`
- Package binary into archive
- Upload artifact

**Job 2: `release`**
- Download all build artifacts
- Create GitHub Release with tag name
- Attach all archives
- Generate SHA256 checksums

**Job 3: `homebrew`**
- Calculate SHA256 for macOS and Linux tarballs
- Generate Homebrew formula
- Push to `dgerlanc/homebrew-tap`

## Homebrew Tap

**Repository:** `dgerlanc/homebrew-tap`

**Formula location:** `Formula/gig.rb`

**Installation:** `brew install dgerlanc/tap/gig`

The formula uses `on_macos`/`on_linux` blocks with architecture-specific URLs and checksums.

## Release Script

**Location:** `scripts/release.sh`

**Usage:** `./scripts/release.sh 0.2.0`

**Behavior:**
1. Validate version argument provided
2. Check for uncommitted changes (abort if dirty)
3. Update version in `Cargo.toml`
4. Run `cargo check` to verify compilation
5. Commit with message "Release v0.2.0"
6. Create git tag `v0.2.0`
7. Push commit and tag to origin

## Setup Requirements

### One-time setup

1. Create `dgerlanc/homebrew-tap` repository on GitHub
2. Create Personal Access Token (PAT) with `repo` scope
3. Add PAT as secret `HOMEBREW_TAP_TOKEN` in gig repo
4. Update `Cargo.toml` edition from `"2024"` to `"2021"`

### Files to create

- `.github/workflows/release.yml` - Release workflow
- `scripts/release.sh` - Release script
- `dgerlanc/homebrew-tap` repo with `Formula/gig.rb` template

## User Experience

```bash
# Release a new version
./scripts/release.sh 0.1.0

# Users install via Homebrew
brew install dgerlanc/tap/gig
```
