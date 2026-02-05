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
git add Cargo.toml Cargo.lock
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
