#!/bin/bash
# Usage: ./scripts/bump_version.sh <new_version>

set -e

NEW_VERSION="$1"

if [ -z "$NEW_VERSION" ]; then
    echo "Error: No version specified."
    echo "Usage: $0 <new_version>"
    exit 1
fi

echo "Bumping version to $NEW_VERSION..."

# 1. Update Cargo.toml
if [ -f "Cargo.toml" ]; then
    # Use sed to replace the version line. Assumes standard formatting.
    # This matches 'version = "x.y.z"' inside the [package] block usually at the top.
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
    echo "Updated Cargo.toml"
else
    echo "Warning: Cargo.toml not found."
fi

# 2. Update CHANGELOG.md
if [ -f "CHANGELOG.md" ]; then
    DATE=$(date +%Y-%m-%d)
    # Replace [Unreleased] with the new version and date, and add a new [Unreleased] section above it.
    sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n## [$NEW_VERSION] - $DATE/" CHANGELOG.md
    echo "Updated CHANGELOG.md"
else
    echo "Warning: CHANGELOG.md not found."
fi

echo "Done! Please verify changes and commit."
echo "  git add Cargo.toml CHANGELOG.md"
echo "  git commit -m \"chore: bump version to $NEW_VERSION\""