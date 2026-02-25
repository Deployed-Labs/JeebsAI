#!/bin/bash

REPO_URL="https://github.com/Deployed-Labs/JeebsAI.git"
REPO_DIR="/root/JeebsAI"

# Clone repo if not present
if [ ! -d "$REPO_DIR" ]; then
    git clone "$REPO_URL" "$REPO_DIR"
fi

cd "$REPO_DIR" || exit 1

# Stash any local changes
git stash

# Pull latest changes from main
git checkout main
git pull origin main

# Build (Rust)
if ! command -v cargo >/dev/null 2>&1; then
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    source $HOME/.cargo/env
fi

cargo build --release

echo "Build complete. Binary is at target/release/jeebs"
