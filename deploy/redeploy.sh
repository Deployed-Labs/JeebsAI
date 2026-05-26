#!/bin/bash
# Redeploy script - pulls latest code and reinstalls dependencies
set -e

cd /opt/jeebsai

echo "==> Pulling latest code..."
git pull origin main

echo "==> Re-running installer..."
bash ./install.sh

if systemctl list-units --full -all | grep -q "^jeebsai.service"; then
    echo "==> Restarting jeebsai.service..."
    sudo systemctl restart jeebsai
fi

echo "==> Waiting for health check..."
sleep 3
curl -sf http://localhost:8000/health && echo "" && echo "✅ Backend is up!"

echo ""
echo "==> Testing login directly..."
curl -s -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"1090mb","password":"password123?!321"}' | python3 -m json.tool

echo ""
echo "✅ Done! Admin login: 1090mb / password123?!321"
