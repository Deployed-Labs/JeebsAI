import re
import os
APP_DIR = os.environ.get("APP_DIR", "/root/JeebsAI")
path = os.path.join(APP_DIR, "home.html")
with open(path, "r") as f:
    content = f.read()

# This is a bulletproof, simplified login script
new_script = """
    <script>
    function login(u, p) {
        console.log("Attempting login for:", u);
        var xhr = new XMLHttpRequest();
        xhr.open("POST", "/api/login", true);
        xhr.setRequestHeader("Content-Type", "application/json");
        xhr.onreadystatechange = function () {
            if (xhr.readyState === 4) {
                if (xhr.status === 200) {
                    alert("Success! Welcome to Jeebs.");
                    var data = JSON.parse(xhr.responseText);
                    localStorage.setItem("token", data.token);
                    location.reload();
                } else {
                    alert("Login Failed (Status " + xhr.status + "): " + xhr.responseText);
                }
            }
        };
        xhr.onerror = function() {
            alert("Network Error: Request blocked or Server unreachable.");
        };
        xhr.send(JSON.stringify({ username: u, password: p }));
    }
    </script>
"""

# Replace everything between the script tags
content = re.sub(r"<script>.*?</script>", new_script, content, flags=re.DOTALL)
with open(path, "w") as f:
    f.write(content)
