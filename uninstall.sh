#!/bin/bash

# JeebsAI Uninstallation Script
# This script stops the application and optionally removes data

echo "=================================================="
echo "   JeebsAI Uninstallation Script"
echo "=================================================="
echo ""

# Find and stop any running gunicorn processes
echo "Checking for running JeebsAI processes..."
PIDS=$(pgrep -f "gunicorn.*app.app:app")

if [ -n "$PIDS" ]; then
    echo "Found running processes: $PIDS"
    echo "Stopping gunicorn processes..."
    kill $PIDS 2>/dev/null || true
    sleep 2
    
    # Check if processes are still running
    PIDS=$(pgrep -f "gunicorn.*app.app:app")
    if [ -n "$PIDS" ]; then
        echo "Force killing remaining processes..."
        kill -9 $PIDS 2>/dev/null || true
    fi
    echo "✓ Processes stopped"
else
    echo "✓ No running processes found"
fi

# Find and stop any Flask development server
FLASK_PIDS=$(pgrep -f "flask.*app.app")
if [ -n "$FLASK_PIDS" ]; then
    echo "Found Flask development server: $FLASK_PIDS"
    echo "Stopping Flask processes..."
    kill $FLASK_PIDS 2>/dev/null || true
    sleep 1
    kill -9 $FLASK_PIDS 2>/dev/null || true
    echo "✓ Flask processes stopped"
fi

echo ""
echo "Application stopped successfully"
echo ""

# Ask about data deletion
read -p "Do you want to remove the virtual environment? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "venv" ]; then
        echo "Removing virtual environment..."
        rm -rf venv
        echo "✓ Virtual environment removed"
    else
        echo "✓ Virtual environment not found"
    fi
fi

echo ""
read -p "Do you want to remove the database? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Respect DATABASE_PATH from .env
    if [ -f ".env" ]; then
        DB_PATH=$(grep "^DATABASE_PATH" .env 2>/dev/null | cut -d'=' -f2 | xargs)
    fi
    DB_FILE="${DB_PATH:-jeebs.db}"
    if [ -f "$DB_FILE" ]; then
        echo "Backing up database to ${DB_FILE}.backup..."
        cp "$DB_FILE" "${DB_FILE}.backup"
        echo "Removing database..."
        rm "$DB_FILE"
        echo "✓ Database removed (backup saved as ${DB_FILE}.backup)"
    else
        echo "✓ Database file not found"
    fi
fi

echo ""
read -p "Do you want to remove the .env file? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -f ".env" ]; then
        echo "Backing up .env to .env.backup..."
        cp .env .env.backup
        echo "Removing .env file..."
        rm .env
        echo "✓ .env file removed (backup saved as .env.backup)"
    else
        echo "✓ .env file not found"
    fi
fi

echo ""
echo "=================================================="
echo "   Uninstallation Complete"
echo "=================================================="
echo ""
echo "The following items were preserved:"
echo "  - Source code"
echo "  - requirements.txt"
if [ -f "jeebs.db.backup" ]; then
    echo "  - Database backup (jeebs.db.backup)"
fi
if [ -f ".env.backup" ]; then
    echo "  - Environment backup (.env.backup)"
fi
echo ""
echo "To completely remove JeebsAI, delete this directory:"
echo "  rm -rf $(pwd)"
echo ""
