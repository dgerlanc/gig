# Homebrew Release Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Automate building and releasing gig binaries via GitHub Actions with Homebrew tap distribution.

**Architecture:** GitHub Actions workflow triggered by version tags builds cross-platform binaries, creates releases, and updates a personal Homebrew tap. A release script handles version bumping and tagging.

**Tech Stack:** GitHub Actions, Rust cross-compilation, Homebrew formula DSL, Bash scripting

---

### Task 1: Fix Cargo.toml Edition

The `edition = "2024"` requires nightly Rust. Change to stable `"2021"`.

**Files:**
- Modify: `Cargo.toml:4`

**Step 1: Update edition**

Change line 4 from:
```toml
edition = "2024"
```
to:
```toml
edition = "2021"
```

**Step 2: Verify it builds**

Run: `cargo build`
Expected: Successful build

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "Change Rust edition from 2024 to 2021"
```

---

### Task 2: Create the Homebrew Tap Repository

Create `dgerlanc/homebrew-tap` on GitHub with initial README.

**Step 1: Create the repository**

Run: `gh repo create homebrew-tap --public --description "Homebrew tap for personal projects" --clone=false`
Expected: Repository created at `dgerlanc/homebrew-tap`

**Step 2: Clone and initialize**

Run:
```bash
cd /tmp && rm -rf homebrew-tap
git clone git@github.com:dgerlanc/homebrew-tap.git
cd homebrew-tap
mkdir -p Formula
```

**Step 3: Create README**

Create `README.md`:
```markdown
# Homebrew Tap

Personal Homebrew tap for my projects.

## Installation

```bash
brew tap dgerlanc/tap
brew install <formula>
```

## Available Formulae

- `gig` - Generate .gitignore files from GitHub's template collection
```

**Step 4: Create placeholder formula**

Create `Formula/gig.rb`:
```ruby
# This formula is auto-updated by GitHub Actions on release.
# Manual edits will be overwritten.

class Gig < Formula
  desc "Generate .gitignore files from GitHub's template collection"
  homepage "https://github.com/dgerlanc/gig"
  version "0.0.0"
  license "Apache-2.0"

  on_macos do
    on_intel do
      url "https://github.com/dgerlanc/gig/releases/download/v0.0.0/gig-v0.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_arm do
      url "https://github.com/dgerlanc/gig/releases/download/v0.0.0/gig-v0.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/dgerlanc/gig/releases/download/v0.0.0/gig-v0.0.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "gig"
  end

  test do
    system "#{bin}/gig", "--help"
  end
end
```

**Step 5: Commit and push**

```bash
git add README.md Formula/gig.rb
git commit -m "Initial tap setup with gig formula placeholder"
git push origin main
```

**Step 6: Return to gig repo**

```bash
cd /Users/dgerlanc/code/gig
```

---

### Task 3: Create Release Script

Create a script that bumps version, commits, tags, and pushes.

**Files:**
- Create: `scripts/release.sh`

**Step 1: Create scripts directory**

Run: `mkdir -p scripts`

**Step 2: Create release script**

Create `scripts/release.sh`:
```bash
#!/usr/bin/env bash
set -euo pipefail

# Release script for gig
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.1.0

VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

# Validate version format (simple check for x.y.z)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format x.y.z (e.g., 0.1.0)"
    exit 1
fi

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

# Check we're on main branch
BRANCH=$(git branch --show-current)
if [[ "$BRANCH" != "main" ]]; then
    echo "Warning: You're on branch '$BRANCH', not 'main'. Continue? [y/N]"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Check if tag already exists
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "Error: Tag v$VERSION already exists."
    exit 1
fi

echo "Releasing version $VERSION..."

# Update version in Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Verify it compiles
echo "Running cargo check..."
cargo check --quiet

# Commit
git add Cargo.toml
git commit -m "Release v$VERSION"

# Tag
git tag "v$VERSION"

# Push
echo "Pushing to origin..."
git push origin "$BRANCH"
git push origin "v$VERSION"

echo ""
echo "Released v$VERSION!"
echo "GitHub Actions will now build and publish the release."
echo "Watch progress at: https://github.com/dgerlanc/gig/actions"
```

**Step 3: Make executable**

Run: `chmod +x scripts/release.sh`

**Step 4: Commit**

```bash
git add scripts/release.sh
git commit -m "Add release script"
```

---

### Task 4: Create GitHub Actions Release Workflow

Create the workflow that builds, releases, and updates Homebrew.

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Create release workflow**

Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package (Unix)
        if: runner.os != 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../gig-${{ github.ref_name }}-${{ matrix.target }}.tar.gz gig
          cd ../../..

      - name: Package (Windows)
        if: runner.os == 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../gig-${{ github.ref_name }}-${{ matrix.target }}.zip gig.exe
          cd ../../..

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: gig-${{ matrix.target }}
          path: gig-${{ github.ref_name }}-${{ matrix.target }}.*

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}

    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true

      - name: Generate checksums
        run: |
          cd artifacts
          sha256sum gig-* > checksums.txt
          cat checksums.txt

      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            artifacts/gig-*
            artifacts/checksums.txt
          generate_release_notes: true

  homebrew:
    name: Update Homebrew Tap
    needs: release
    runs-on: ubuntu-latest

    steps:
      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true

      - name: Calculate checksums
        id: checksums
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          echo "version=$VERSION" >> $GITHUB_OUTPUT

          echo "sha256_x86_64_apple=$(sha256sum artifacts/gig-${{ github.ref_name }}-x86_64-apple-darwin.tar.gz | cut -d' ' -f1)" >> $GITHUB_OUTPUT
          echo "sha256_aarch64_apple=$(sha256sum artifacts/gig-${{ github.ref_name }}-aarch64-apple-darwin.tar.gz | cut -d' ' -f1)" >> $GITHUB_OUTPUT
          echo "sha256_x86_64_linux=$(sha256sum artifacts/gig-${{ github.ref_name }}-x86_64-unknown-linux-gnu.tar.gz | cut -d' ' -f1)" >> $GITHUB_OUTPUT

      - name: Checkout homebrew-tap
        uses: actions/checkout@v4
        with:
          repository: dgerlanc/homebrew-tap
          token: ${{ secrets.HOMEBREW_TAP_TOKEN }}
          path: homebrew-tap

      - name: Update formula
        run: |
          cat > homebrew-tap/Formula/gig.rb << 'FORMULA'
          class Gig < Formula
            desc "Generate .gitignore files from GitHub's template collection"
            homepage "https://github.com/dgerlanc/gig"
            version "${{ steps.checksums.outputs.version }}"
            license "Apache-2.0"

            on_macos do
              on_intel do
                url "https://github.com/dgerlanc/gig/releases/download/${{ github.ref_name }}/gig-${{ github.ref_name }}-x86_64-apple-darwin.tar.gz"
                sha256 "${{ steps.checksums.outputs.sha256_x86_64_apple }}"
              end
              on_arm do
                url "https://github.com/dgerlanc/gig/releases/download/${{ github.ref_name }}/gig-${{ github.ref_name }}-aarch64-apple-darwin.tar.gz"
                sha256 "${{ steps.checksums.outputs.sha256_aarch64_apple }}"
              end
            end

            on_linux do
              on_intel do
                url "https://github.com/dgerlanc/gig/releases/download/${{ github.ref_name }}/gig-${{ github.ref_name }}-x86_64-unknown-linux-gnu.tar.gz"
                sha256 "${{ steps.checksums.outputs.sha256_x86_64_linux }}"
              end
            end

            def install
              bin.install "gig"
            end

            test do
              system "#{bin}/gig", "--help"
            end
          end
          FORMULA

      - name: Commit and push
        run: |
          cd homebrew-tap
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add Formula/gig.rb
          git commit -m "Update gig to ${{ steps.checksums.outputs.version }}"
          git push
```

**Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "Add release workflow for cross-platform builds"
```

---

### Task 5: Document PAT Setup (Manual Step)

The user needs to create a PAT and add it as a secret. This is documented, not automated.

**Step 1: Update README with release instructions**

Add to `README.md` before the closing section:

```markdown
## Installation

### Homebrew (macOS/Linux)

```bash
brew install dgerlanc/tap/gig
```

### From GitHub Releases

Download the appropriate binary from the [releases page](https://github.com/dgerlanc/gig/releases).
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "Add installation instructions to README"
```

**Step 3: Manual - Create PAT**

Go to https://github.com/settings/tokens and create a new Personal Access Token (classic) with `repo` scope. Name it `HOMEBREW_TAP_TOKEN`.

**Step 4: Manual - Add secret**

Go to https://github.com/dgerlanc/gig/settings/secrets/actions and add a new secret:
- Name: `HOMEBREW_TAP_TOKEN`
- Value: (paste the PAT)

---

### Task 6: Push Changes and Verify

**Step 1: Push all commits**

```bash
git push origin main
```

**Step 2: Verify CI passes**

Check https://github.com/dgerlanc/gig/actions - the CI workflow should pass.

---

### Task 7: Test Release (Optional First Release)

Once PAT is configured, test the full release flow.

**Step 1: Run release script**

```bash
./scripts/release.sh 0.1.0
```

**Step 2: Monitor release workflow**

Watch https://github.com/dgerlanc/gig/actions for the Release workflow.

**Step 3: Verify Homebrew tap updated**

Check https://github.com/dgerlanc/homebrew-tap/blob/main/Formula/gig.rb has the new version and checksums.

**Step 4: Test installation**

```bash
brew tap dgerlanc/tap
brew install gig
gig --help
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Fix Cargo.toml edition to 2021 |
| 2 | Create homebrew-tap repository |
| 3 | Create release script |
| 4 | Create release workflow |
| 5 | Document PAT setup + update README |
| 6 | Push and verify CI |
| 7 | Test release (after PAT configured) |
