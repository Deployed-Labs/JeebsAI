#!/bin/bash
#
# Fixes "non-monotonic index" errors caused by macOS metadata files (._*)
# appearing inside the .git directory on external drives.
#

echo "🧹 Cleaning up macOS metadata from .git directory..."
find .git -name "._*" -print -delete
echo "✅ Done. Git should work now."