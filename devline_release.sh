#!/bin/bash
set -e

echo "🚀 Starting Devline1 Release Workflow..."

# Optional: Cleanup temporary fix logs
echo "------------------------------------------------"
echo "🧹 Cleanup"
read -p "Delete temporary fix logs (*_FIXED.md, *_VERIFICATION.md, etc)? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -f *_FIXED.md *_FIXED_*.md *_VERIFICATION.md FIX_CHECKLIST.md CORTEX_MISSING_FIXED.md CHAT_405_FIX.md FINAL_COMPILATION_FIXES.md DUPLICATE_BRACES_FIXED_2.md EXTRA_BRACE_FIXED.md BRAIN_PARSING_API_FIXED.md
    echo "✅ Temporary logs deleted."
fi

# 1. Commit to devline1
echo "------------------------------------------------"
echo "📦 Processing devline1..."
if git show-ref --verify --quiet refs/heads/devline1; then
    git checkout devline1
    git pull origin devline1 || echo "⚠️ Could not pull devline1 (might be local only)"
else
    git checkout -b devline1
fi

git add .
git commit -m "Update devline1: $(date '+%Y-%m-%d %H:%M:%S')" || echo "Nothing to commit"
git push -u origin devline1

# 2. Merge to main (Simulating PR merge)
echo "------------------------------------------------"
echo "🔀 Merging devline1 into main..."
git checkout main
git pull origin main
if ! git merge devline1 --no-edit; then
    echo "❌ Merge conflict! Please resolve conflicts manually, then commit and push."
    exit 1
fi
git push origin main

# 3. Release
echo "------------------------------------------------"
echo "🏷️ Create Release"
read -p "Enter version tag (e.g. v1.2.0): " VERSION

if [ -n "$VERSION" ]; then
    git tag -a "$VERSION" -m "Release $VERSION"
    git push origin "$VERSION"
    echo "✅ Release $VERSION pushed successfully!"
else
    echo "⚠️ No version provided. Skipping release tag."
fi

echo "------------------------------------------------"
echo "🎉 Workflow Complete. Returning to devline1..."
git checkout devline1