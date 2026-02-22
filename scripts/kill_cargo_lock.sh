#!/usr/bin/env bash
# kill_cargo_lock.sh - Kill all cargo processes and remove build directory lock

set -e

# Kill any running cargo processes
pkill -9 cargo || true

# Remove cargo build lock file if it exists
rm -f target/.cargo-lock

echo "All cargo processes killed and build lock removed. You can now run cargo build safely."
