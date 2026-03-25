import sqlite3
import json
from datetime import datetime
from pathlib import Path

DB_PATH = Path('/data/jeebs.db')

def get_db():
    """Get database connection"""
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    return conn

def dict_from_row(row):
    """Convert sqlite3.Row to dict"""
    return dict(row) if row else None

def init_db():
    """Initialize database schema"""
    conn = get_db()
    cursor = conn.cursor()
    
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT UNIQUE NOT NULL,
        email TEXT UNIQUE NOT NULL,
        password_hash TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        is_admin INTEGER DEFAULT 0
    )
    ''')
    
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS conversations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        title TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(user_id) REFERENCES users(id)
    )
    ''')
    
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        conversation_id INTEGER NOT NULL,
        role TEXT NOT NULL,
        content TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(conversation_id) REFERENCES conversations(id)
    )
    ''')
    
    # Create indexes for improved query performance
    cursor.execute('CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)')
    cursor.execute('CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)')
    cursor.execute('CREATE INDEX IF NOT EXISTS idx_conversations_user_id ON conversations(user_id)')
    cursor.execute('CREATE INDEX IF NOT EXISTS idx_conversations_created_at ON conversations(created_at)')
    cursor.execute('CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id)')
    cursor.execute('CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at)')
    
    conn.commit()
    conn.close()

class User:
    @staticmethod
    def create(username, email, password_hash):
        conn = get_db()
        cursor = conn.cursor()
        try:
            cursor.execute(
                'INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)',
                (username, email, password_hash)
            )
            conn.commit()
            return cursor.lastrowid
        finally:
            conn.close()
    
    @staticmethod
    def get_by_username(username):
        conn = get_db()
        cursor = conn.cursor()
        cursor.execute('SELECT * FROM users WHERE username = ?', (username,))
        row = cursor.fetchone()
        conn.close()
        return dict_from_row(row)
    
    @staticmethod
    def get_by_id(user_id):
        conn = get_db()
        cursor = conn.cursor()
        cursor.execute('SELECT * FROM users WHERE id = ?', (user_id,))
        row = cursor.fetchone()
        conn.close()
        return dict_from_row(row)
    
    @staticmethod
    def get_all():
        conn = get_db()
        cursor = conn.cursor()
        cursor.execute('SELECT id, username, email, is_admin, created_at FROM users')
        rows = cursor.fetchall()
        conn.close()
        return [dict(row) for row in rows]

class Conversation:
    @staticmethod
    def create(user_id, title='New Chat'):
        conn = get_db()
        cursor = conn.cursor()
        try:
            cursor.execute(
                'INSERT INTO conversations (user_id, title) VALUES (?, ?)',
                (user_id, title)
            )
            conn.commit()
            return cursor.lastrowid
        finally:
            conn.close()
    
    @staticmethod
    def get_by_id(conv_id):
        conn = get_db()
        cursor = conn.cursor()
        cursor.execute('SELECT * FROM conversations WHERE id = ?', (conv_id,))
        row = cursor.fetchone()
        conn.close()
        return dict_from_row(row)
    
    @staticmethod
    def get_user_conversations(user_id, page=1, per_page=20):
        """Get paginated conversations for a user"""
        conn = get_db()
        cursor = conn.cursor()
        
        # Get total count
        cursor.execute(
            'SELECT COUNT(*) as count FROM conversations WHERE user_id = ?',
            (user_id,)
        )
        total = cursor.fetchone()['count']
        
        # Get paginated results
        offset = (page - 1) * per_page
        cursor.execute(
            'SELECT * FROM conversations WHERE user_id = ? ORDER BY updated_at DESC LIMIT ? OFFSET ?',
            (user_id, per_page, offset)
        )
        rows = cursor.fetchall()
        conn.close()
        
        return {
            'items': [dict(row) for row in rows],
            'total': total,
            'page': page,
            'per_page': per_page,
            'pages': (total + per_page - 1) // per_page
        }
    
    @staticmethod
    def update_title(conv_id, title):
        conn = get_db()
        cursor = conn.cursor()
        try:
            cursor.execute(
                'UPDATE conversations SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?',
                (title, conv_id)
            )
            conn.commit()
        finally:
            conn.close()

class Message:
    @staticmethod
    def create(conversation_id, role, content):
        conn = get_db()
        cursor = conn.cursor()
        try:
            cursor.execute(
                'INSERT INTO messages (conversation_id, role, content) VALUES (?, ?, ?)',
                (conversation_id, role, content)
            )
            conn.commit()
            return cursor.lastrowid
        finally:
            conn.close()
    
    @staticmethod
    def get_conversation_messages(conv_id, page=1, per_page=50):
        """Get paginated messages for a conversation"""
        conn = get_db()
        cursor = conn.cursor()
        
        # Get total count
        cursor.execute(
            'SELECT COUNT(*) as count FROM messages WHERE conversation_id = ?',
            (conv_id,)
        )
        total = cursor.fetchone()['count']
        
        # Get paginated results
        offset = (page - 1) * per_page
        cursor.execute(
            'SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC LIMIT ? OFFSET ?',
            (conv_id, per_page, offset)
        )
        rows = cursor.fetchall()
        conn.close()
        
        return {
            'items': [dict(row) for row in rows],
            'total': total,
            'page': page,
            'per_page': per_page,
            'pages': (total + per_page - 1) // per_page
        }
