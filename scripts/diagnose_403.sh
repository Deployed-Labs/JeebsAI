#!/usr/bin/env bash
set -euo pipefail

# Diagnose 403 after login.
# - Check service status and logs.
# - Verify /webui/ is accessible.
# - Check webui directory permissions.
# - Test the static route.

SERVICE_NAME=${SERVICE_NAME:-"jeebs"}
APP_DIR=${APP_DIR:-"/root/JeebsAI"}
PORT=${PORT:-"8080"}

echo "=== JeebsAI 403 Diagnosis ==="
echo ""

echo "1. Service Status:"
systemctl status "$SERVICE_NAME" --no-pager || true

echo ""
echo "2. Recent Logs:"
sudo journalctl -u "$SERVICE_NAME" -n 20 --no-pager -l || true

echo ""
echo "3. WebUI Directory Permissions:"
ls -la "$APP_DIR/webui" | head -20 || true

echo ""
echo "4. Test /webui/ Endpoint:"
curl -I "http://127.0.0.1:${PORT}/webui/" || echo "Failed to connect"

echo ""
echo "5. Test /webui/index.html:"
curl -I "http://127.0.0.1:${PORT}/webui/index.html" || echo "Failed to connect"

echo ""
echo "6. Check Static Route in Binary:"
grep -n 'Files::new("/webui"' "$APP_DIR/target/release/jeebs" >/dev/null 2>&1 && echo "✓ Static route found" || echo "✗ Static route NOT found"
