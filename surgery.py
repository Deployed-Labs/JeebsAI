import re

with open("src/main.rs", "r") as f:
    lines = f.readlines()

# 1. Remove any previous broken .service() lines we added
new_lines = [l for l in lines if "jeebs::auth::" not in l]

# 2. Find the HttpServer block and inject correctly
final_lines = []
for line in new_lines:
    final_lines.append(line)
    if "App::new()" in line:
        # We inject the services with the exact indentation needed
        final_lines.append('            .service(jeebs::auth::login)\n')
        final_lines.append('            .service(jeebs::auth::login_pgp)\n')
        final_lines.append('            .service(jeebs::auth::change_password)\n')

with open("src/main.rs", "w") as f:
    f.writelines(final_lines)
