#!/bin/bash
APP_DIR=${APP_DIR:-"/root/JeebsAI"}
cd "$APP_DIR"
# This finds the fetch/axios call and ensures it uses the relative path
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|url: "/login"|url: "/api/login"|g' {} +
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|"/login"|"/api/login"|g' {} +
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|fetch("/login"|fetch("/api/login"|g' {} +
echo "✅ Frontend JS endpoints updated to /api/login."
