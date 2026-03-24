from flask import Flask, jsonify

app = Flask(__name__)


@app.route('/')
def index():
    return jsonify({"status": "ok", "message": "Hello from Dockerized JeebsAI skeleton"})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8000)
