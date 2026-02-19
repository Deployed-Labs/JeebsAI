#!/bin/bash
cd /root/JeebsAI

# This looks for your script tag and adds a version number (v=2)
# Example: script.js becomes script.js?v=2
sed -i 's/\.js"/.js?v=2"/g' index.html

echo "✅ Cache buster applied. The phone will now force-load the new code."
