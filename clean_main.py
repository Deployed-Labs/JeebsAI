import re

with open("src/main.rs", "r") as f:
    content = f.read()

# Remove all previous broken service injections to start fresh
content = re.sub(r'\.service\(jeebs::auth::.*?\)\n', '', content)

# Inject the services cleanly after App::new()
services = (
    '            .service(jeebs::auth::login)\n'
    '            .service(jeebs::auth::login_pgp)\n'
    '            .service(jeebs::auth::change_password)\n'
)
content = content.replace('App::new()\n', f'App::new()\n{services}')

with open("src/main.rs", "w") as f:
    f.write(content)
