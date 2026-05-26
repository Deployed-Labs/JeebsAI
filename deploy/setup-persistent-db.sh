#!/bin/bash

# Setup script for persistent database on a direct VPS (non-Docker)
# Ensures the database path is configured and initialized

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}🔧 Setting up persistent database for JeebsAI...${NC}"

cd /opt/jeebsai

# Ensure .env exists
if [ ! -f ".env" ]; then
    echo -e "${RED}❌ .env file not found. Run ./install.sh first.${NC}"
    exit 1
fi

# Create data directory if using a custom path
source .env 2>/dev/null || true
DB_FILE="${DATABASE_PATH:-./jeebs.db}"

DB_DIR=$(dirname "$DB_FILE")
if [ "$DB_DIR" != "." ]; then
    echo -e "${YELLOW}Creating database directory: $DB_DIR${NC}"
    mkdir -p "$DB_DIR"
    chmod 755 "$DB_DIR"
fi

# Initialize database
echo -e "${YELLOW}Initializing database...${NC}"
source venv/bin/activate
python3 -c "from app.models import init_db, ensure_admin; init_db(); ensure_admin(); print('Done')"

# Verify database exists
if [ -f "$DB_FILE" ]; then
    echo -e "${GREEN}✅ Database file exists at $DB_FILE${NC}"
    ls -lh "$DB_FILE"
else
    echo -e "${RED}❌ Database file not found at $DB_FILE${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Database setup complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Database location: $DB_FILE"
echo ""
echo "To backup:"
echo "  cp $DB_FILE ${DB_FILE}.backup"
echo ""
