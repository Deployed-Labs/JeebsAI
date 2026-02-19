#!/bin/bash
cd /root/JeebsAI
# This injects a simple error handler into the script
sed -i 's/fetch(.*)/fetch(url, options).catch(err => alert("Connection Error: " + err))/' home.html
echo "✅ Error alerts enabled for mobile debugging."
