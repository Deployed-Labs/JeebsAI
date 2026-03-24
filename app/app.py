from flask import Flask, jsonify, send_from_directory, render_template_string
from flask_cors import CORS
import os
from .models import init_db
from .auth import auth_bp, token_required, admin_required
from .chat import chat_bp
from .admin import admin_bp

app = Flask(__name__, static_folder='../static', static_url_path='/static')
CORS(app)

# Initialize database
init_db()

# Register blueprints
app.register_blueprint(auth_bp)
app.register_blueprint(chat_bp)
app.register_blueprint(admin_bp)

# Admin panel route (client-side token validation via admin.html)
@app.route('/admin', methods=['GET'])
def admin_panel():
    """Serve admin panel (token validated by client-side JavaScript)"""
    try:
        with open(os.path.join(app.static_folder, 'admin.html'), 'r') as f:
            return f.read()
    except:
        return jsonify({"message": "Admin panel not found"}), 404

# Health check
@app.route('/health', methods=['GET'])
def health():
    return jsonify({"status": "ok", "service": "JeebsAI"}), 200

# Serve main HTML
@app.route('/', methods=['GET'])
def index():
    try:
        with open(os.path.join(app.static_folder, 'index.html'), 'r') as f:
            return f.read()
    except:
        return jsonify({"message": "JeebsAI Chat API - see /static/index.html"}), 200

# Serve static files
@app.route('/static/<path:path>')
def serve_static(path):
    return send_from_directory(app.static_folder, path)

# API info
@app.route('/api', methods=['GET'])
def api_info():
    return jsonify({
        "service": "JeebsAI",
        "version": "1.0.0",
        "endpoints": {
            "auth": "/api/auth/*",
            "chat": "/api/chat/*",
            "admin": "/api/admin/*"
        }
    }), 200

@app.errorhandler(404)
def not_found(error):
    return jsonify({"message": "Endpoint not found"}), 404

@app.errorhandler(500)
def server_error(error):
    return jsonify({"message": "Internal server error"}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8000, debug=False)
