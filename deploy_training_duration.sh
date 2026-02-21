#!/usr/bin/env bash
set -e

chmod +x PUSH_TRAINING_DURATION.sh

git add -A

git commit -m "COMPLETE: Training duration limits (1-5 minutes)

✅ Minimum: 1 minute (60 seconds)
✅ Maximum: 5 minutes (300 seconds)

HOW IT WORKS:

1. Start training cycle
2. Crawl websites (check max)
3. Research topics (check max)
4. Check minimum:
   • If < 1 min: continue exploring
   • If >= 1 min: ready to finish
5. Check maximum:
   • If >= 5 min: stop immediately
6. Finalize and report

BENEFITS:

• Consistent learning duration
• Auto-extends if quick
• Hard max at 5 minutes
• Efficient exploration
• Time-bounded cycles

Ready for production!" || echo "Already staged"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TRAINING DURATION LIMITS DEPLOYED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Next: bash update.sh on VPS"
echo ""
echo "Training cycles will:"
echo "  • Minimum: 1 minute"
echo "  • Maximum: 5 minutes"
echo "  • Auto-extend if quick"
echo "  • Hard-stop at max"
echo ""
echo "⏱️ Ready to learn with time boundaries!"
echo ""
