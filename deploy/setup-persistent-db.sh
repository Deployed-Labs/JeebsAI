#!/bin/bash

# Setup script for persistent database with bind mount
# Run this on your VPS to ensure database persists across restarts

set -e

echo "🔧 Setting up persistent database for JeebsAI..."

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create data directory
echo -e "${YELLOW}Creating /opt/jeebsai/data directory...${NC}"
mkdir -p /opt/jeebsai/data
chmod 755 /opt/jeebsai/data

# Verify we're in the right directory
if [ ! -f "docker-compose.prod.yml" ]; then
    echo -e "${RED}❌ Error: docker-compose.prod.yml not found!${NC}"
    echo "Please run this script from /opt/jeebsai directory:"
    echo "cd /opt/jeebsai && bash deploy/setup-persistent-db.sh"
    exit 1
fi

echo -e "${GREEN}✅ Data directory ready${NC}"

# Stop current containers
echo -e "${YELLOW}Stopping current containers...${NC}"
docker compose -f deploy/docker-compose.prod.yml down || true

# Check if old database exists in Docker volume (if migrating from volume)
echo -e "${YELLOW}Checking for existing database to migrate...${NC}"
if docker volume ls | grep -q "deploy_jeebs_data"; then
    echo -e "${YELLOW}Found existing Docker volume 'deploy_jeebs_data'${NC}"
    
    # Check if database file exists in our bind mount
    if [ ! -f "/opt/jeebsai/data/jeebs.db" ]; then
        echo -e "${YELLOW}Attempting to copy data from Docker volume...${NC}"
        docker run --rm -v deploy_jeebs_data:/volume -v /opt/jeebsai/data:/hostdata \
            alpine cp /volume/jeebs.db /hostdata/jeebs.db 2>/dev/null || true
        
        if [ -f "/opt/jeebsai/data/jeebs.db" ]; then
            echo -e "${GREEN}✅ Database migrated from Docker volume${NC}"
        fi
    fi
fi

# Rebuild and start containers
echo -e "${YELLOW}Rebuilding and starting containers...${NC}"
docker compose -f deploy/docker-compose.prod.yml up -d --build

# Wait for web service to be ready
echo -e "${YELLOW}Waiting for JeebsAI to start (this may take 30 seconds)...${NC}"
sleep 10

# Check health
for i in {1..12}; do
    if curl -f http://localhost:8000/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ JeebsAI is running and healthy!${NC}"
        break
    fi
    echo -n "."
    sleep 5
    if [ $i -eq 12 ]; then
        echo -e "${RED}❌ JeebsAI did not start properly${NC}"
        echo "Check logs with: docker compose -f deploy/docker-compose.prod.yml logs web"
        exit 1
    fi
done

# Verify database exists
if [ -f "/opt/jeebsai/data/jeebs.db" ]; then
    echo -e "${GREEN}✅ Database file exists at /opt/jeebsai/data/jeebs.db${NC}"
    ls -lh /opt/jeebsai/data/jeebs.db
else
    echo -e "${YELLOW}⚠️  Database file will be created on first use${NC}"
fi

# Show summary
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Database location: /opt/jeebsai/data/"
echo "This is a bind mount - data persists across restarts"
echo ""
echo "To view logs:"
echo "  docker compose -f deploy/docker-compose.prod.yml logs -f web"
echo ""
echo "To backup database:"
echo "  cp /opt/jeebsai/data/jeebs.db /opt/jeebsai/data/jeebs.db.backup"
echo ""
echo "To restart in future:"
echo "  cd /opt/jeebsai"
echo "  docker compose -f deploy/docker-compose.prod.yml restart"
echo ""
