#!/bin/bash

# JeebsAI Login Fix Script — Run this on your VPS (non-Docker)
# Diagnoses and fixes login issues by resetting the admin account

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== JeebsAI Login Diagnostic & Fix ===${NC}"
echo ""

cd /opt/jeebsai

# 1. Check service status
echo -e "${YELLOW}1. Checking JeebsAI service...${NC}"
sudo systemctl status jeebsai --no-pager || true
echo ""

# 2. Check health endpoint
echo -e "${YELLOW}2. Checking health endpoint...${NC}"
if curl -s http://localhost:8000/health; then
    echo -e "\n${GREEN}✅ Backend is responding${NC}"
else
    echo -e "${RED}❌ Backend is not responding${NC}"
fi
echo ""

# 3. Check database
echo -e "${YELLOW}3. Checking database...${NC}"
source .env 2>/dev/null || true
DB_FILE="${DATABASE_PATH:-./jeebs.db}"
if [ -f "$DB_FILE" ]; then
    ls -lh "$DB_FILE"
    echo -e "${GREEN}✅ Database exists${NC}"
else
    echo -e "${RED}❌ Database not found at $DB_FILE${NC}"
fi
echo ""

# 4. Reset admin account
echo -e "${YELLOW}4. Resetting admin account...${NC}"
source venv/bin/activate
python3 -c "
from app.models import init_db, ensure_admin
init_db()
ensure_admin()
print('✅ Admin account has been reset')
"
echo ""

# 5. Test login
echo -e "${YELLOW}5. Testing login...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"1090mb","password":"password123?!321"}')

echo "Login response:"
echo "$LOGIN_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$LOGIN_RESPONSE"

if echo "$LOGIN_RESPONSE" | grep -q "Login successful"; then
    echo -e "${GREEN}✅ Login successful!${NC}"
else
    echo -e "${RED}❌ Login failed — check logs with: sudo journalctl -u jeebsai --no-pager -n 50${NC}"
fi

echo ""
echo -e "${GREEN}=== Diagnostic Complete ===${NC}"

