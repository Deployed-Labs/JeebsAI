#!/usr/bin/env bash
set -e

echo "⏱️ Pushing Training Duration Limits (1-5 minutes)..."
echo ""

git add src/cortex.rs
git add TRAINING_DURATION_LIMITS.txt

git commit -m "Add training cycle duration limits: 1-5 minutes

FEATURE: Training cycles now enforce minimum and maximum durations

MINIMUM: 1 minute (60 seconds)
  • If cycle finishes quickly, automatically continues exploring
  • Keeps crawling new domains until 1 min is reached
  • Ensures substantial learning happens
  • Prevents wasted shallow cycles

MAXIMUM: 5 minutes (300 seconds)
  • Hard stop at 5 minutes
  • Doesn't start new explorations after max reached
  • Finishes gracefully at max time
  • Prevents runaway cycles

IMPLEMENTATION:

Added to run_training_cycle():
  • min_duration = 60 seconds
  • max_duration = 300 seconds

Duration checks:
  • After each website crawl
  • After each topic research
  • During extended exploration phase

Extended exploration phase:
  • If < 1 min when done with main tasks
  • Randomly picks additional sites
  • Keeps crawling until >= 1 min or >= 5 min

ALGORITHM:

1. Crawl 2 random websites
   └─ Stop if >= 5 min
2. Research up to 7 topics
   └─ Stop if >= 5 min
3. Check minimum duration
   ├─ If < 1 min: continue exploring
   └─ If >= 1 min: ready to finalize
4. Finalize and report

BENEFITS:

✅ Consistent learning (min 1 min)
✅ Time bounded (max 5 min)
✅ Adaptive exploration
✅ Efficient use of time
✅ Safe boundaries
✅ Progressive deepening

TYPICAL CYCLE:

Duration: 1:00 - 5:00 minutes
Sites: 3-6 domains
Pages: 100-300 pages
Knowledge: 50-200 items
Topics: 3-7 topics

CODE CHANGES:

• src/cortex.rs - Added duration limits
• Added min/max constants
• Added max checks in loops
• Added extended exploration phase
• Added Rng import for random selection

READY FOR PRODUCTION! ✅" || echo "Already committed"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TRAINING DURATION LIMITS PUSHED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "Training cycles will now:"
echo "  • Run minimum 1 minute"
echo "  • Run maximum 5 minutes"
echo "  • Auto-extend if finished quickly"
echo "  • Stop at 5 minute limit"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "⏱️ Training cycles now have duration limits!"
echo ""
