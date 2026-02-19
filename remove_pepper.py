import re

path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    content = f.read()

# 1. Force the pepper to be an empty string
content = re.sub(r'let pepper = .*?;', 'let pepper = "";', content)

# 2. Ensure the verify call isn't using a secret key elsewhere
# (Looking for common argon2 verification patterns)
content = content.replace('verify_password(&req.password, &user.password, &pepper)', 
                          'verify_password(&req.password, &user.password, "")')

with open(path, "w") as f:
    f.write(content)
