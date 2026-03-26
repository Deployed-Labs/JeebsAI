#!/bin/bash

# JeebsAI Login Fix Script - Run this on your VPS
# This script will diagnose and fix any login issues

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== JeebsAI Login Diagnostic & Fix ===${NC}"
echo ""

cd /opt/jeebsai

# 1. Check Docker status
echo -e "${YELLOW}1. Checking Docker containers...${NC}"
docker compose -f deploy/docker-compose.prod.yml ps
echo ""

# 2. Check health endpoint
echo -e "${YELLOW}2. Checking health endpoint...${NC}"
if curl -s http://localhost:8000/health; then
    echo -e "${GREEN}✅ Backend is responding${NC}"
else
    echo -e "${RED}❌ Backend is not responding${NC}"
fi
echo ""

# 3. Check database
echo -e "${YELLOW}3. Checking database file...${NC}"
if [ -f "/opt/jeebsai/data/jeebs.db" ]; then
    ls -lh /opt/jeebsai/data/jeebs.db
    echo -e "${GREEN}✅ Database exists${NC}"
else
    echo -e "${RED}❌ Database does not exist - will be created on first run${NC}"
fi
echo ""

# 4. Check recent logs
echo -e "${YELLOW}4. Recent logs (last 20 lines)...${NC}"
docker compose -f deploy/docker-compose.prod.yml logs web --tail=20
echo ""

# 5. Create admin user
echo -e "${YELLOW}5. Creating/verifying admin user...${NC}"

# Create a temporary Python script
cat > /tmp/create_admin.py << 'PYTHONSCRIPT'
from app.models import User, init_db
from werkzeug.security import generate_password_hash
import sqlite3

# Initialize database
init_db()

# Check if admin exists
admin = User.get_by_username('admin')

if admin:
    print(f"✅ Admin user already exists (ID: {admin['id']})")
    print(f"   Username: {admin['username']}")
    print(f"   Email: {admin['email']}")
    print(f"   Is Admin: {bool(admin['is_admin'])}")
else:
    # Create admin user
    password_hash = generate_password_hash('admin')
    user_id = User.create('admin', 'admin@jeebs.club', password_hash)
    
    # Make user admin
    conn = sqlite3.connect('/data/jeebs.db')
    cursor = conn.cursor()
    cursor.execute('UPDATE users SET is_admin = 1 WHERE id = ?', (user_id,))
    conn.commit()
    conn.close()
    
    print(f"✅ Created admin user (ID: {user_id})")
    print(f"   Username: admin")
    print(f"   Password: admin")
    print(f"   Email: admin@jeebs.club")

# List all users
print("\nAll users in database:")
cursor = sqlite3.connect('/data/jeebs.db').cursor()
cursor.execute("SELECT id, username, email, is_admin FROM users")
for row in cursor.fetchall():
    print(f"  - ID: {row[0]}, Username: {row[1]}, Email: {row[2]}, Admin: {bool(row[3])}")
PYTHONSCRIPT

# Run the script in the container
docker compose -f deploy/docker-compose.prod.yml exec -T web python /tmp/create_admin.py
rm /tmp/create_admin.py

echo ""

# 6. Test login
echo -e "${YELLOW}6. Testing login...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}')

echo "Login response:"
echo "$LOGIN_RESPONSE" | python -m json.tool 2>/dev/null || echo "$LOGIN_RESPONSE"

if echo "$LOGIN_RESPONSE" | grep -q "Login successful"; then
    echo -e "${GREEN}✅ Login successful!${NC}"
else
    echo -e "${RED}❌ Login failed${NC}"
fi

echo ""
echo -e "${GREEN}=== Diagnostic Complete ===${NC}"
echo ""
echo "If login still fails:"
echo "1. Check the logs above for errors"
echo "2. Try accessing the web UI at: http://your-domain.com"
echo "3. Try username 'admin' and password 'admin'"
echo ""
