#!/usr/bin/env bash
set -euo pipefail

# Usage: ./deploy_jeebs.sh [REPO_DIR] [BRANCH]
# REPO_DIR default: /root/JeebsAI
# BRANCH default: ${BRANCH:-main}

REPO_DIR="${1:-/root/JeebsAI}"
BRANCH="${2:-${BRANCH:-main}}"
REPO_URL="${REPO_URL:-git@github.com:Deployed-Labs/JeebsAI.git}"
FORCE="${3:-${FORCE:-false}}"

echo "Deploy target: $REPO_DIR (branch: $BRANCH)"

if [ ! -d "$REPO_DIR" ]; then
  echo "Repo directory not found — cloning $REPO_URL into $REPO_DIR"
  git clone "$REPO_URL" "$REPO_DIR"
fi

cd "$REPO_DIR"

# Ensure origin exists and points to the expected remote
if ! git remote get-url origin >/dev/null 2>&1; then
  git remote add origin "$REPO_URL"
fi

echo "Fetching remotes..."
git fetch --all --prune

# Force-align local branch to remote branch (creates if needed)
echo "Checking out and aligning branch '$BRANCH' with origin/$BRANCH"
git checkout -B "$BRANCH" "origin/$BRANCH" || git checkout -B "$BRANCH"

# Safety: prompt before destructive reset unless FORCE set to true
if [ "${FORCE,,}" != "true" ]; then
  read -r -p "About to reset local branch '$BRANCH' to origin/$BRANCH (this will discard local changes). Continue? (y/N): " yn
  case "$yn" in
    [Yy]* ) echo "Proceeding with reset..." ;;
    * ) echo "Aborting deploy." ; exit 1 ;;
  esac
else
  echo "Force mode enabled; skipping confirmation prompt."
fi

git reset --hard "origin/$BRANCH" || true

# Ensure Rust toolchain exists
if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing rustup/cargo..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Prefer sccache if available to speed up rebuilds
if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER="$(command -v sccache)"
  echo "Using sccache at $RUSTC_WRAPPER"
else
  echo "sccache not found; install with 'cargo install sccache' for faster builds."
fi

export DATABASE_URL="${DATABASE_URL:-sqlite:jeebs.db}"
# Build using SQLX offline metadata to avoid needing a live DB during compile
export SQLX_OFFLINE=1

echo "Building release..."
cargo build --release

echo "Stopping any running instance..."
pkill -f 'target/release/jeebs' || true

echo "Starting Jeebs in background..."
nohup ./target/release/jeebs > /var/log/jeebs.log 2>&1 &

if command -v ufw >/dev/null 2>&1; then
  ufw allow 8080 || true
fi

echo "Done. Tail logs with: tail -F /var/log/jeebs.log"
