#!/bin/bash
cd /root/JeebsAI

# Remove any duplicate service lines we might have accidentally added
sed -i '/jeebs::auth::login/d' src/main.rs

# Find the line with App::new() and add our services immediately after it
sed -i '/App::new()/a \            .service(jeebs::auth::login)\n            .service(jeebs::auth::login_pgp)\n            .service(jeebs::auth::change_password)' src/main.rs

echo "🔄 Restarting Jeebs..."
sudo systemctl restart jeebs
