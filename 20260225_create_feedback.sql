-- Migration: Create feedback table for JeebsAI chat learning
CREATE TABLE IF NOT EXISTS feedback (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_message_id INTEGER NOT NULL,
    username TEXT,
    feedback_type TEXT NOT NULL, -- 'up', 'down', 'correction'
    feedback_text TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(chat_message_id) REFERENCES chat_history(id)
);
