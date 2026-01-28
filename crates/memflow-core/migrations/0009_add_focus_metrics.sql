CREATE TABLE IF NOT EXISTS focus_metrics (
    timestamp INTEGER PRIMARY KEY,
    apm INTEGER NOT NULL,
    window_switch_count INTEGER NOT NULL,
    focus_score REAL NOT NULL
);

