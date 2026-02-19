import re

path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    content = f.read()

# 1. Replace the verify_password call with a direct equality check
# We change: verify_password(&req.password, &user.password, &pepper)
# To: req.password == user.password
content = re.sub(
    r'verify_password\(&req\.password, &user\.password, &pepper\)', 
    'req.password == user.password', 
    content
)

# 2. In case the function call looks slightly different (no pepper)
content = re.sub(
    r'verify_password\(&req\.password, &user\.password, ""\)', 
    'req.password == user.password', 
    content
)

with open(path, "w") as f:
    f.write(content)
