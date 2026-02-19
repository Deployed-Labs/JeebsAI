import re

with open("src/main.rs", "r") as f:
    content = f.read()

# This replaces the entire block from App::new() to the end of the .bind()
pattern = r"App::new\(\).*?\.bind\(\(\"0\.0\.0\.0\", port\)\)\?\s*\.run\(\)\?\s*\.await\s*}"
replacement = """App::new()
            .app_data(web::Data::new(state.clone()))
            .service(jeebs::auth::login)
            .service(jeebs::auth::login_pgp)
            .service(jeebs::auth::change_password)
            .service(jeebs::cortex::admin_train)
            .service(jeebs::cortex::admin_crawl)
            .service(jeebs::cortex::search_brain)
            .service(jeebs::cortex::reindex_brain)
            .service(jeebs::cortex::visualize_brain)
            .service(jeebs::cortex::get_logic_graph)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}"""

new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)

with open("src/main.rs", "w") as f:
    f.write(new_content)
