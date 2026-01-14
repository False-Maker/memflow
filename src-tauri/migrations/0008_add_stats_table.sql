CREATE TABLE IF NOT EXISTS recording_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL, -- 'YYYY-MM-DD'
    reason TEXT NOT NULL, -- 'privacy_mode', 'blocklist'
    count INTEGER DEFAULT 0,
    UNIQUE(date, reason)
);
