import os
import re

APP_DIR = os.environ.get("APP_DIR", "/root/JeebsAI")
path = os.path.join(APP_DIR, "src/auth/mod.rs")
with open(path, "r") as f:
    lines = f.readlines()

# We are going to look for the lines we messed up and clean them
new_lines = []
skip = False

for line in lines:
    # If we see the broken comment or the rate limit error, skip it
    if 'Too many login attempts' in line or 'rate_limit_key' in line:
        continue
    # If the line is just a stray bracket/comment from our previous attempt
    if '}*/),' in line:
        continue
    new_lines.append(line)

with open(path, "w") as f:
    f.writelines(new_lines)
