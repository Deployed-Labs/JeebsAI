#!/usr/bin/env bash
# update_master.sh - Pull latest from GitHub and restart JeebsAI from /root/JeebsAI

set -e
cd /root/JeebsAI

echo "[update_master] Pulling latest from GitHub..."
git pull origin main

echo "[update_master] Restarting JeebsAI service..."
systemctl restart jeebs || echo "[update_master] Warning: Could not restart jeebs service. Please check manually."

echo "[update_master] Done!"
