#!/bin/bash

# JeebsAI Status Check Script
# This script checks the status of the application and its components

echo "=================================================="
echo "   JeebsAI Status Check"
echo "=================================================="
echo ""

# Check for Python
echo "🐍 Python:"
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version)
    echo "  ✓ $PYTHON_VERSION"
else
    echo "  ❌ Python 3 not found"
fi

echo ""

# Check for virtual environment
echo "📦 Virtual Environment:"
if [ -d "venv" ]; then
    echo "  ✓ Virtual environment exists"
    if [ -f "venv/bin/activate" ]; then
        echo "  ✓ Activation script found"
    fi
else
    echo "  ❌ Virtual environment not found"
    echo "     Run ./install.sh to create it"
fi

echo ""

# Check for .env file
echo "⚙️  Configuration:"
if [ -f ".env" ]; then
    echo "  ✓ .env file exists"
    
    # Check for required variables
    if grep -q "SECRET_KEY" .env; then
        echo "  ✓ SECRET_KEY is set"
    else
        echo "  ⚠️  SECRET_KEY not found in .env"
    fi
    
    if grep -q "FLASK_ENV" .env; then
        FLASK_ENV=$(grep "FLASK_ENV" .env | cut -d'=' -f2)
        echo "  ✓ FLASK_ENV=$FLASK_ENV"
    fi
    
    if grep -q "PORT" .env; then
        PORT=$(grep "PORT" .env | cut -d'=' -f2)
        echo "  ✓ PORT=$PORT"
    fi
else
    echo "  ❌ .env file not found"
    echo "     Run ./install.sh to create it"
fi

echo ""

# Check for database
echo "🗄️  Database:"
if [ -f "jeebs.db" ]; then
    DB_SIZE=$(du -h jeebs.db | cut -f1)
    echo "  ✓ Database exists ($DB_SIZE)"
    
    # Count records if sqlite3 is available
    if command -v sqlite3 &> /dev/null; then
        USER_COUNT=$(sqlite3 jeebs.db "SELECT COUNT(*) FROM users;" 2>/dev/null || echo "?")
        CONV_COUNT=$(sqlite3 jeebs.db "SELECT COUNT(*) FROM conversations;" 2>/dev/null || echo "?")
        MSG_COUNT=$(sqlite3 jeebs.db "SELECT COUNT(*) FROM messages;" 2>/dev/null || echo "?")
        echo "  ✓ Users: $USER_COUNT | Conversations: $CONV_COUNT | Messages: $MSG_COUNT"
    fi
else
    echo "  ⚠️  Database not found"
    echo "     Will be created on first run"
fi

echo ""

# Check for running processes
echo "🚀 Application Status:"
GUNICORN_PIDS=$(pgrep -f "gunicorn.*app.app:app" 2>/dev/null)
FLASK_PIDS=$(pgrep -f "flask.*app.app" 2>/dev/null)

if [ -n "$GUNICORN_PIDS" ]; then
    echo "  ✓ Gunicorn is RUNNING (PIDs: $GUNICORN_PIDS)"
    
    # Show process info
    for PID in $GUNICORN_PIDS; do
        if command -v ps &> /dev/null; then
            PS_INFO=$(ps -p $PID -o etime= -o %cpu= -o %mem= 2>/dev/null)
            if [ -n "$PS_INFO" ]; then
                UPTIME=$(echo $PS_INFO | awk '{print $1}')
                CPU=$(echo $PS_INFO | awk '{print $2}')
                MEM=$(echo $PS_INFO | awk '{print $3}')
                echo "    PID $PID: Uptime=$UPTIME CPU=${CPU}% MEM=${MEM}%"
            fi
        fi
    done
    
    # Try to check if port is listening
    if [ -f ".env" ] && grep -q "PORT" .env; then
        PORT=$(grep "PORT" .env | cut -d'=' -f2)
        if command -v netstat &> /dev/null; then
            if netstat -tuln 2>/dev/null | grep -q ":$PORT "; then
                echo "  ✓ Listening on port $PORT"
            fi
        elif command -v ss &> /dev/null; then
            if ss -tuln 2>/dev/null | grep -q ":$PORT "; then
                echo "  ✓ Listening on port $PORT"
            fi
        fi
    fi
    
elif [ -n "$FLASK_PIDS" ]; then
    echo "  ✓ Flask dev server is RUNNING (PIDs: $FLASK_PIDS)"
else
    echo "  ⏸️  Application is NOT RUNNING"
    echo ""
    echo "  To start in development mode:"
    echo "    source venv/bin/activate"
    echo "    python3 -m flask --app app.app run --host 0.0.0.0 --port 8000"
    echo ""
    echo "  To start in production mode:"
    echo "    source venv/bin/activate"
    echo "    gunicorn -w 4 -b 0.0.0.0:8000 app.app:app"
fi

echo ""

# Check system resources
echo "💻 System Resources:"
if command -v free &> /dev/null; then
    MEM_INFO=$(free -h | grep "Mem:" | awk '{print "Used: "$3" / Total: "$2}')
    echo "  Memory: $MEM_INFO"
fi

if command -v df &> /dev/null; then
    DISK_INFO=$(df -h . | tail -1 | awk '{print "Used: "$3" / Total: "$2" ("$5" full)"}')
    echo "  Disk: $DISK_INFO"
fi

if command -v uptime &> /dev/null; then
    LOAD=$(uptime | awk -F'load average:' '{print $2}')
    echo "  Load Average:$LOAD"
fi

echo ""

# Admin credentials reminder
echo "🔐 Admin Credentials:"
echo "  Username: 1090mb"
echo "  Password: password123?!321"

echo ""
echo "=================================================="
echo ""

# Suggest next actions
if [ -z "$GUNICORN_PIDS" ] && [ -z "$FLASK_PIDS" ]; then
    echo "💡 Next steps: Start the application with one of the commands above"
elif [ -f ".env" ] && grep -q "PORT" .env; then
    PORT=$(grep "PORT" .env | cut -d'=' -f2)
    echo "💡 Access the application at: http://localhost:$PORT"
fi

echo ""
