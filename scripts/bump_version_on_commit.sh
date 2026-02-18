#!/usr/bin/env bash
set -euo pipefail

# Auto-increment patch (X.Y.Z -> X.Y.(Z+1)) after a normal commit.
# - Skips when last commit is already a bump commit.
# - Writes both Cargo.toml and VERSION (VERSION uses 'v' prefix).
# - Commits with [skip ci] to avoid CI loops.

# Allow opt-out
if [ -n "${SKIP_VERSION_BUMP:-}" ]; then
  exit 0
fi

last_msg=$(git log -1 --pretty=%B || true)
case "$last_msg" in
  *"chore(release): bump version to v"*|*"[skip version-bump]"*)
    # Already a bump commit — do nothing
    exit 0
    ;;
esac

# Get current version from Cargo.toml
current=$(sed -n 's/^version = "\([0-9]\+\.[0-9]\+\.[0-9]\+\)"/\1/p' Cargo.toml | head -n1)
if [ -z "$current" ]; then
  echo "Error: could not detect version in Cargo.toml" >&2
  exit 1
fi
IFS='.' read -r major minor patch <<< "$current"
patch=$((patch + 1))
next="$major.$minor.$patch"

# Update Cargo.toml and VERSION
sed -E -i.bak "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$next\"/" Cargo.toml
rm -f Cargo.toml.bak
printf "v%s\n" "$next" > VERSION

# Stage and commit (skip CI to avoid loops). If commit fails (e.g. nothing changed), ignore.
git add Cargo.toml VERSION || true
if git commit -m "chore(release): bump version to v$next [skip ci]"; then
  # Push only if branch has an upstream configured
  branch=$(git rev-parse --abbrev-ref HEAD)
  if git rev-parse --abbrev-ref --symbolic-full-name @{u} >/dev/null 2>&1; then
    git push origin "$branch" || true
  fi
fi
