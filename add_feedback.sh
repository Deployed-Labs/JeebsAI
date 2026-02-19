#!/bin/bash
cd /root/JeebsAI
# Adds a simple script to change button text to "Connecting..." when clicked
sed -i '/<button/s/Login/Login" onclick="this.innerText='\''Connecting...'\''"/' index.html
echo "✅ Visual feedback added to Login button."
