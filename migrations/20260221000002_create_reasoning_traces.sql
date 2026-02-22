-- Migration: create reasoning_traces table
CREATE TABLE IF NOT EXISTS reasoning_traces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    username TEXT,
    prompt TEXT NOT NULL,
    response TEXT NOT NULL,
    metadata TEXT
);
