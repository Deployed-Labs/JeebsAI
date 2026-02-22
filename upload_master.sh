#!/usr/bin/env bash
# upload_master.sh - Commit and push changes from /root/JeebsAI to GitHub main branch

set -e
cd /Users/shoup/Documents/GitHub/JeebsAI

echo "[upload_master] Adding all changes..."
git add .

if ! git diff --cached --quiet; then
  echo "[upload_master] Committing changes..."
  git commit -m "Update from upload_master.sh"
else
  echo "[upload_master] No changes to commit."
fi

echo "[upload_master] Pushing to GitHub..."
git push origin main

echo "[upload_master] Done!"}},{