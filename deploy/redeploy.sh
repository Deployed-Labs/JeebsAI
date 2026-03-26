#!/bin/bash
# Full redeploy script - pulls latest code and rebuilds Docker image
set -e

cd /opt/jeebsai

echo "==> Pulling latest code..."
git pull origin main

echo "==> Stopping containers..."
docker compose -f deploy/docker-compose.prod.yml down

echo "==> Rebuilding image (no cache)..."
docker compose -f deploy/docker-compose.prod.yml build --no-cache web

echo "==> Starting containers..."
docker compose -f deploy/docker-compose.prod.yml up -d

echo "==> Waiting for health check..."
sleep 8
curl -sf http://localhost:8000/health && echo "" && echo "✅ Backend is up!"

echo ""
echo "==> Container status:"
docker compose -f deploy/docker-compose.prod.yml ps

echo ""
echo "==> Recent logs:"
docker compose -f deploy/docker-compose.prod.yml logs web --tail=15

echo ""
echo "✅ Redeploy complete. Login at https://jeebs.club"
echo "   Username: admin"
echo "   Password: admin"
