#!/bin/bash
cd /root/JeebsAI

# Backup main.rs just in case
cp src/main.rs src/main.rs.bak

# Inject the login services right before change_password
sed -i '/.service(auth::change_password)/i \            .service(auth::login)\n            .service(auth::login_pgp)' src/main.rs

echo "✅ Routes injected. Rebuilding and restarting Jeebs..."

# Restart the service (which triggers the build if you use cargo run in your service)
sudo systemctl restart jeebs
