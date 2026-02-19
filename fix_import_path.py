path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    lines = f.readlines()

with open(path, "w") as f:
    for line in lines:
        if "use crate::admin::user::User;" in line or "use crate::models::User;" in line:
            # We try the most common path for a registered module
            f.write("use crate::models::User;\n")
        else:
            f.write(line)
