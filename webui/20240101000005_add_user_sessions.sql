CREATE TABLE IF NOT EXISTS user_sessions (
    username TEXT PRIMARY KEY,
    ip TEXT,
    user_agent TEXT,
    last_seen TEXT
);