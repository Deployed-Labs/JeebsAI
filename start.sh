#!/bin/bash

# JeebsAI Startup Script
# Usage: ./start.sh

set -e

# Ensure we are in the script's directory
cd "$(dirname "$0")"

echo "--- Starting JeebsAI ---"

# Build release binary
echo "[1/2] Building release binary..."
cargo build --release

# Run binary
echo "[2/2] Launching server..."
if [ "$EUID" -ne 0 ]; then
    echo "Note: Running with sudo to bind port 80..."
    sudo ./target/release/jeebs
else
    ./target/release/jeebs
fi