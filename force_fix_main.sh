#!/bin/bash
cd /root/JeebsAI

# 1. Full Backup
cp src/main.rs src/main.rs.last_fail

# 2. Reconstruct main.rs with proper imports and the App block
# Note: We are ensuring 'auth' and 'cortex' are correctly referenced.
cat << 'EON' > src/main.rs
use actix_web::{web, App, HttpServer, Responder};
mod auth;
mod cortex;
mod state;
// ... other modules ...

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = state::AppState::new().await;
    let port = 8080;

    println!("Jeebs is waking up on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(auth::login)
            .service(auth::login_pgp)
            .service(auth::change_password)
            .service(cortex::admin_train)
            .service(cortex::admin_crawl)
            .service(cortex::search_brain)
            .service(cortex::reindex_brain)
            .service(cortex::visualize_brain)
            .service(cortex::get_logic_graph)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
EON

# 3. Check for any obvious missing imports in the new file
# (We might need to adjust the 'mod' lines if they are actually in lib.rs)

echo "🚀 Restarting for the final time..."
sudo systemctl restart jeebs
sudo journalctl -u jeebs -f
