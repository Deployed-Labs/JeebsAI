#!/bin/bash

# JeebsAI Login Fix Script - Run this on your VPS
set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== JeebsAI Login Diagnostic & Fix ===${NC}"
echo ""

cd /opt/jeebsai

echo -e "${YELLOW}1. Checking health endpoint...${NC}"
if curl -s http://localhost:8000/health; then
    echo -e "${GREEN}✅ Backend is responding${NC}"
else
    echo -e "${RED}❌ Backend is not responding${NC}"
fi
echo ""

echo -e "${YELLOW}2. Ensuring admin account exists...${NC}"
source venv/bin/activate
python3 - << 'PYTHONSCRIPT'
from app.models import init_db, ensure_admin, DB_PATH

init_db()
ensure_admin()
print(f"✅ Admin ensured in database: {DB_PATH}")
print("   Username: 1090mb")
print("   Password: password123?!321")
PYTHONSCRIPT
echo ""

echo -e "${YELLOW}3. Testing login...${NC}"
if [ -n "$JEEBSAI_ADMIN_USERNAME" ] && [ -n "$JEEBSAI_ADMIN_PASSWORD" ]; then
    LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8000/api/auth/login \
      -H "Content-Type: application/json" \
      -d "{\"username\":\"$JEEBSAI_ADMIN_USERNAME\",\"password\":\"$JEEBSAI_ADMIN_PASSWORD\"}")
    
    echo "Login response:"
    echo "$LOGIN_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$LOGIN_RESPONSE"
    
    if echo "$LOGIN_RESPONSE" | grep -q "Login successful"; then
        echo -e "${GREEN}✅ Login successful!${NC}"
    else
        echo -e "${RED}❌ Login failed${NC}"
    fi
else
    echo -e "${YELLOW}Skipping login test. Set JEEBSAI_ADMIN_USERNAME and JEEBSAI_ADMIN_PASSWORD to test automatically.${NC}"
fi

echo ""
echo -e "${GREEN}=== Diagnostic Complete ===${NC}"
