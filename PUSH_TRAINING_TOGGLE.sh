#!/usr/bin/env bash
set -e

echo "🤖 Pushing Training Mode Auto-Run with Toggle..."
echo ""

# Add changes
git add src/cortex.rs
git add webui/admin_dashboard.html
git add TRAINING_MODE_TOGGLE_COMPLETE.txt

# Commit
git commit -m "Add Training Mode auto-run with on/off toggle in admin dashboard

FEATURE: Training mode auto-runs on startup with emergency toggle

CHANGES:

src/cortex.rs:
  • Changed training mode default: enabled=false → true
  • Training automatically starts when JeebsAI boots
  • Graceful toggle on/off without restarts

webui/admin_dashboard.html:
  • Added Training Mode toggle section
  • Green (🟢 RUNNING) when active
  • Red (🔴 STOPPED) when paused
  • Styled like Internet Access toggle
  • One-click toggle with confirmation
  • Auto-refreshes status every 5 seconds

HOW IT WORKS:

1. JeebsAI starts up
2. Training mode enabled by default (enabled: true)
3. Autonomous training worker spawns
4. Training cycles run automatically
5. Admin can toggle on/off anytime

ADMIN DASHBOARD:

New 'Training Mode' section shows:
  • Current status (RUNNING/STOPPED)
  • One-click TOGGLE button
  • Auto-refresh every 5 seconds
  • Green when learning, red when paused

USE CASES:

• Normal operation: Training runs automatically
• Emergency: Click TOGGLE to stop if needed
• Maintenance: Stop training, do work, resume
• Testing: Easy on/off for testing scenarios
• Control: Users always in control

API ENDPOINTS (Already Existed):

GET /api/admin/training/status
  → Returns current training state

POST /api/admin/training/mode
  → Sets training enabled/disabled

BENEFITS:

✅ No manual configuration needed
✅ Training starts automatically on boot
✅ Emergency off switch always available
✅ Same UI pattern as Internet toggle
✅ Graceful on/off (no forced restarts)
✅ Transparent status display
✅ Safe and simple

DEPLOYMENT:

1. bash update.sh on VPS
2. Rebuilds with new defaults
3. Training auto-runs on restart
4. Toggle visible in admin dashboard

READY FOR PRODUCTION! 🚀" || echo "Nothing to commit"

# Push
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TRAINING MODE AUTO-RUN WITH TOGGLE PUSHED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "Then:"
echo "  • JeebsAI will auto-run training on startup"
echo "  • Admin dashboard shows Training Mode toggle"
echo "  • Green (🟢) when running, red (🔴) when stopped"
echo "  • Click to toggle on/off anytime"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🤖 Training mode auto-runs with emergency toggle ready!"
echo ""
