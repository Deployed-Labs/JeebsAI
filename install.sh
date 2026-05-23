#!/bin/bash

# JeebsAI Installation Script
# This script installs dependencies and starts the application

set -e

echo "=================================================="
echo "   JeebsAI Installation Script"
echo "=================================================="
echo ""

# Check if Python 3 is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: Python 3 is not installed"
    echo "Please install Python 3.8 or higher and try again"
    exit 1
fi

PYTHON_VERSION=$(python3 --version | cut -d' ' -f2 | cut -d'.' -f1,2)
echo "✓ Found Python $PYTHON_VERSION"

# Check if pip is installed
if ! command -v pip3 &> /dev/null; then
    echo "❌ Error: pip3 is not installed"
    echo "Please install pip3 and try again"
    exit 1
fi
echo "✓ Found pip3"

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo ""
    echo "Creating virtual environment..."
    python3 -m venv venv
    echo "✓ Virtual environment created"
else
    echo "✓ Virtual environment already exists"
fi

# Activate virtual environment
echo ""
echo "Activating virtual environment..."
source venv/bin/activate

# Upgrade pip
echo ""
echo "Upgrading pip..."
pip install --upgrade pip

# Install dependencies
echo ""
echo "Installing dependencies from requirements.txt..."
pip install -r requirements.txt
echo "✓ Dependencies installed"

# Create .env file if it doesn't exist
if [ ! -f ".env" ]; then
    echo ""
    echo "Creating .env file..."
    cat > .env << EOF
# JeebsAI Environment Configuration
# Generated on $(date)

# Flask Configuration
FLASK_ENV=production
SECRET_KEY=$(python3 -c "import secrets; print(secrets.token_hex(32))")

# Server Configuration
HOST=0.0.0.0
PORT=8000

# Database Configuration
DATABASE_PATH=./jeebs.db

# Admin Credentials (hardcoded in code)
# Username: 1090mb
# Password: password123?!321
EOF
    echo "✓ .env file created with a secure SECRET_KEY"
    echo ""
    echo "⚠️  IMPORTANT: Review and edit .env file if needed"
else
    echo "✓ .env file already exists"
fi

# Initialize database
echo ""
echo "Initializing database..."
python3 -c "from app.models import init_db, ensure_admin; init_db(); ensure_admin(); print('✓ Database initialized')"

# Check if gunicorn is available
if ! command -v gunicorn &> /dev/null; then
    echo ""
    echo "❌ Error: gunicorn not found after installation"
    echo "Please check the installation logs above"
    exit 1
fi

echo ""
echo "=================================================="
echo "   Installation Complete!"
echo "=================================================="
echo ""
echo "To start the application, run one of:"
echo ""
echo "  Development mode (Flask debug):"
echo "    source venv/bin/activate"
echo "    python3 -m flask --app app.app run --host 0.0.0.0 --port 8000"
echo ""
echo "  Production mode (Gunicorn):"
echo "    source venv/bin/activate"
echo "    gunicorn -w 4 -b 0.0.0.0:8000 app.app:app"
echo ""
echo "Or create a systemd service for automatic startup."
echo ""
echo "Admin credentials:"
echo "  Username: 1090mb"
echo "  Password: password123?!321"
echo ""
echo "Access the application at: http://localhost:8000"
echo ""
