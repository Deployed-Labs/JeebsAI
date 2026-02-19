#!/bin/bash
cd /root/JeebsAI
echo "🚀 Starting background rebuild. You can go do other stuff now!"

# Revert to the last known working main.rs before our edits
if [ -f "src/main.rs.bak" ]; then
    cp src/main.rs.bak src/main.rs
fi

# Run the build in the background
nohup cargo build --release > /root/jeebs_build.log 2>&1 &

echo "✅ Build is running in the background. Check progress with: tail -f /root/jeebs_build.log"
