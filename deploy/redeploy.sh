#!/bin/bash
# Redeploy script — pulls latest code and restarts the systemd service
set -e

cd /opt/jeebsai

echo "==> Pulling latest code..."
git pull origin main

echo "==> Installing/updating dependencies..."
source venv/bin/activate
pip install -r requirements.txt

echo "==> Restarting JeebsAI service..."
sudo systemctl restart jeebsai

echo "==> Waiting for health check..."
sleep 5
curl -sf http://localhost:8000/health && echo "" && echo "✅ Backend is up!"

echo ""
echo "==> Service status:"
sudo systemctl status jeebsai --no-pager

echo ""
echo "✅ Done! Login at https://jeebs.club"

