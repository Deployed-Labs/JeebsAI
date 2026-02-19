path = "/root/JeebsAI/home.html"
with open(path, "r") as f:
    content = f.read()

xhr_login = """
        function login(u, p) {
            var xhr = new XMLHttpRequest();
            xhr.open("POST", "/api/login", true);
            xhr.setRequestHeader("Content-Type", "application/json");
            xhr.onreadystatechange = function () {
                if (xhr.readyState === 4) {
                    if (xhr.status === 200) {
                        var data = JSON.parse(xhr.responseText);
                        localStorage.setItem("token", data.token);
                        location.reload();
                    } else {
                        alert("Login Failed: Status " + xhr.status + " - " + xhr.responseText);
                    }
                }
            };
            xhr.onerror = function() {
                alert("XHR Network Error - The request was blocked by the browser.");
            };
            xhr.send(JSON.stringify({ username: u, password: p }));
        }
"""

import re
content = re.sub(r"async function login\(.*?\)\s*\{.*?\}", xhr_login, content, flags=re.DOTALL)
with open(path, "w") as f:
    f.write(content)
