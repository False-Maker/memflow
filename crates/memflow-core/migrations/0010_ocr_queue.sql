-- OCR 任务队列表
CREATE TABLE IF NOT EXISTS ocr_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending', -- pending | processing | done | failed
    retry_count INTEGER DEFAULT 0,
    error_message TEXT,
    created_at INTEGER DEFAULT (strftime('%s','now')),
    updated_at INTEGER DEFAULT (strftime('%s','now')),
    FOREIGN KEY (activity_id) REFERENCES activity_logs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ocr_queue_status ON ocr_queue(status);
CREATE INDEX IF NOT EXISTS idx_ocr_queue_updated ON ocr_queue(updated_at);
