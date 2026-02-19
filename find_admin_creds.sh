#!/bin/bash

DB_PATH="/root/JeebsAI/jeebs.db"

echo "🔍 Searching for admin credentials in $DB_PATH..."

# 1. Check for a users or admin table
echo "--- Table Structure ---"
sqlite3 $DB_PATH ".tables"

# 2. Look for entries in common table names
echo -e "\n--- Potential Admin Entries ---"
# This checks common table names like 'users', 'admins', or 'config'
tables=$(sqlite3 $DB_PATH ".tables")

if [[ $tables == *"users"* ]]; then
    sqlite3 $DB_PATH "SELECT * FROM users WHERE role='admin' OR username='admin';"
elif [[ $tables == *"config"* ]]; then
    sqlite3 $DB_PATH "SELECT * FROM config WHERE key LIKE '%pass%';"
else
    echo "Could not find a standard 'users' table. Dumping all table schemas to look for password fields..."
    sqlite3 $DB_PATH ".schema"
fi

# 3. Check environment variables in the service file
echo -e "\n--- Checking Service Environment ---"
grep "ADMIN" /etc/systemd/system/jeebs.service
grep "PASS" /etc/systemd/system/jeebs.service

# 4. Check for a .env file
if [ -f "/root/JeebsAI/.env" ]; then
    echo -e "\n--- Found .env file ---"
    grep -E "PASS|ADMIN|USER" /root/JeebsAI/.env
fi
