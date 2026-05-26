#!/bin/bash

# Setup script for persistent database path in .env
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}🔧 Configuring persistent database path for JeebsAI...${NC}"

cd /opt/jeebsai
mkdir -p data

if [ ! -f ".env" ]; then
    echo -e "${YELLOW}No .env found. Running installer first...${NC}"
    bash ./install.sh
fi

if grep -q "^DATABASE_PATH=" .env; then
    sed -i 's|^DATABASE_PATH=.*|DATABASE_PATH=./data/jeebs.db|' .env
else
    echo "DATABASE_PATH=./data/jeebs.db" >> .env
fi

if [ -f "jeebs.db" ] && [ ! -f "data/jeebs.db" ]; then
    cp jeebs.db data/jeebs.db
fi

echo -e "${GREEN}✅ DATABASE_PATH configured to ./data/jeebs.db${NC}"
echo "Run ./status.sh after restart to confirm."
