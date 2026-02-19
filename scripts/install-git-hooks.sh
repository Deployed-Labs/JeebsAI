#!/usr/bin/env bash
set -euo pipefail

# Install git hooks for this repository (set core.hooksPath)
# Run this after cloning to enable the post-commit auto-bump hook.

git_root=$(git rev-parse --show-toplevel 2>/dev/null || echo "")
if [ -z "$git_root" ]; then
  echo "Not a git repository. Run this from inside the repository." >&2
  exit 1
fi

git config core.hooksPath .githooks
chmod +x .githooks/post-commit scripts/bump_version_on_commit.sh || true

echo "Git hooks installed (core.hooksPath set to .githooks)."

echo "To disable automatic bumping set: export SKIP_VERSION_BUMP=1"