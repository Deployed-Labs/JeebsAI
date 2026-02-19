#!/bin/bash
cd /root/JeebsAI

# This changes the manual injection to an explicit route mapping
sed -i 's|.service(auth::login)|.service(web::resource("/login").route(web::post().to(auth::login)))|g' src/main.rs
sed -i 's|.service(auth::login_pgp)|.service(web::resource("/login_pgp").route(web::post().to(auth::login_pgp)))|g' src/main.rs

echo "🔄 Re-mapping routes and restarting..."
sudo systemctl restart jeebs

# Wait for rebuild
echo "Waiting 15 seconds for Rust to recompile..."
sleep 15

echo "🧪 Testing /login again..."
curl -X POST http://127.0.0.1:8080/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"Password123!"}' -i
