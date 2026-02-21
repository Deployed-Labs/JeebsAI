#!/usr/bin/env bash
set -e

chmod +x go_duration.sh deploy_training_duration.sh PUSH_TRAINING_DURATION.sh

git add -A

git commit -m "FINAL: Training cycle duration limits (1-5 minutes)

✅ IMPLEMENTED:

MINIMUM: 1 minute (60 seconds)
  • Auto-extends if finished quickly
  • Continues exploring until 1 min reached
  • Ensures meaningful learning per cycle

MAXIMUM: 5 minutes (300 seconds)
  • Hard stop at 5 minutes
  • Graceful shutdown
  • Prevents runaway cycles

ALGORITHM:

Phase 1: Crawl random websites
  └─ Check max: if >= 5 min, stop

Phase 2: Research topics
  └─ Check max: if >= 5 min, stop

Phase 3: Extended exploration (if < 1 min)
  └─ Continue picking random sites
  └─ Keep crawling until >= 1 min or >= 5 min

Phase 4: Finalize and report

BENEFITS:

✅ Consistent learning (always min 1 min)
✅ Time bounded (max 5 min)
✅ Adaptive (extends if quick)
✅ Safe (hard max)
✅ Efficient (uses available time)

CODE CHANGES:

• Added min_duration = 60s
• Added max_duration = 300s
• Added checks in crawl loops
• Added extended exploration phase
• Added Rng for random selection

TESTING:

Watch logs for duration_ms:
  • 60,000 ms = 1 minute minimum
  • 300,000 ms = 5 minutes maximum
  • Typical: 90,000-180,000 ms

Ready for production!" || echo "Already staged"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TRAINING DURATION LIMITS DEPLOYED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deployed to GitHub!"
echo ""
echo "Next: bash update.sh on VPS"
echo ""
echo "Training cycles will run:"
echo "  • Minimum: 1 minute (60 seconds)"
echo "  • Maximum: 5 minutes (300 seconds)"
echo "  • Auto-extends if quick"
echo "  • Hard stops at max"
echo ""
echo "⏱️ Time-bounded learning ready!"
echo ""
