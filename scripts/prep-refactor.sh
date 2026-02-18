#!/usr/bin/env bash
set -euo pipefail

echo "==> Prep: verify environment and run checks"

# Basic environment
command -v rustc >/dev/null || { echo "ERROR: rustc not found on PATH"; exit 1; }
command -v cargo >/dev/null || { echo "ERROR: cargo not found on PATH"; exit 1; }

# Ensure working tree is clean
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: working tree is not clean. Commit or stash changes before refactoring."; git status --porcelain; exit 1
fi

# Formatting, lints, tests
echo "- Running cargo fmt --check"
cargo fmt --all -- --check

echo "- Running cargo clippy (warnings = errors)"
cargo clippy --all-targets --all-features -- -D warnings || { echo "clippy failed"; exit 1; }

echo "- Running cargo test"
cargo test --all --verbose

# Quick secret-scan in repo (grep for common GitHub token patterns)
echo "- Scanning for obvious token patterns (ghp_, GITHUB_TOKEN, GHCR_PAT)"
if git grep -n --break --heading -I "ghp_\|GITHUB_TOKEN\|GHCR_PAT" -- . ":(exclude)scripts/prep-refactor.sh" >/dev/null 2>&1; then
  echo "ERROR: possible secret pattern found in repository. Inspect and remove before continuing."; git grep -n --break --heading -I "ghp_\|GITHUB_TOKEN\|GHCR_PAT" -- . ":(exclude)scripts/prep-refactor.sh"; exit 1
fi

echo "All prep checks passed. Create your refactor branch (example: git checkout -b refactor/your-task) and start coding."