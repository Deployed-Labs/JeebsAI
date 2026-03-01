-- Structured action log table with user association and JSON details
CREATE TABLE IF NOT EXISTS action_logs (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT,
    action TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    severity TEXT NOT NULL DEFAULT 'INFO' CHECK(severity IN ('INFO', 'WARN', 'ERROR')),
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_action_logs_user ON action_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_action_logs_severity ON action_logs(severity);
CREATE INDEX IF NOT EXISTS idx_action_logs_timestamp ON action_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_action_logs_action ON action_logs(action);
