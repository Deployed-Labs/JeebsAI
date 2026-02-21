#!/usr/bin/env bash
set -e

echo "🤖 Final push: Training mode auto-run with toggle..."
echo ""

chmod +x go_training_toggle.sh deploy_training_toggle.sh PUSH_TRAINING_TOGGLE.sh

git add -A

git commit -m "FINAL: Training mode auto-run with on/off toggle

✅ Training auto-runs on JeebsAI startup
✅ On/off toggle in admin dashboard
✅ Same UI style as Internet toggle
✅ Emergency pause/resume control

DEFAULT: Training ENABLED (enabled=true)

ADMIN DASHBOARD:
• Green (🟢 RUNNING) when learning
• Red (🔴 STOPPED) when paused
• One-click TOGGLE button
• Confirmation dialog
• Auto-refreshes status

FILES CHANGED:
• src/cortex.rs - Changed default enabled to true
• webui/admin_dashboard.html - Added toggle UI and functions

DEPLOYMENT:
1. bash update.sh on VPS
2. Training auto-runs on restart
3. Toggle visible in admin dashboard
4. Safe on/off control always available

Ready for production!" || echo "Nothing to commit"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TRAINING MODE TOGGLE DEPLOYED TO GITHUB!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Ready to deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "After update:"
echo "  • Training auto-runs on startup"
echo "  • Admin dashboard shows 🤖 Training Mode toggle"
echo "  • 🟢 RUNNING when learning"
echo "  • 🔴 STOPPED when paused"
echo "  • Click to toggle on/off"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🤖 Training mode ready - auto-run with emergency toggle!"
echo ""
