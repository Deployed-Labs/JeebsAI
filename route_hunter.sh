#!/bin/bash
APP_DIR=${APP_DIR:-"/root/JeebsAI"}
cd "$APP_DIR"
echo "🔍 LISTING ALL REGISTERED ROUTES IN CODE..."

# Look for the App initialization block
grep -A 50 "App::new()" src/main.rs

# Look for where the login might be defined in modules
echo -e "\n🔍 SEARCHING FOR LOGIN HANDLERS..."
grep -rE "async fn.*login" src/
