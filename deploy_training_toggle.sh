#!/usr/bin/env bash
set -e

chmod +x PUSH_TRAINING_TOGGLE.sh

git add -A

git commit -m "COMPLETE: Training Mode auto-run with on/off toggle

✅ Training auto-runs on JeebsAI startup
✅ Admin dashboard has on/off toggle
✅ Same style as Internet Access toggle
✅ Green when running, red when stopped
✅ Emergency pause/resume available

Default behavior: Training ENABLED on startup

To pause: Click TOGGLE in admin dashboard
To resume: Click TOGGLE again

Safe, simple, user-controlled. Ready for production!" || echo "Already staged"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TRAINING MODE AUTO-RUN DEPLOYED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Next: bash update.sh on VPS"
echo ""
echo "After update:"
echo "  • JeebsAI will auto-run training on restart"
echo "  • Admin dashboard shows: 🤖 Training Mode"
echo "  • Toggle to pause/resume learning"
echo ""
echo "🤖 Ready to learn automatically!"
echo ""
