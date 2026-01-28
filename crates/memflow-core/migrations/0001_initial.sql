-- 活动日志表
CREATE TABLE IF NOT EXISTS activity_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    window_title TEXT NOT NULL,
    image_path TEXT NOT NULL,
    ocr_text TEXT,
    phash TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_activity_logs_timestamp ON activity_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_activity_logs_app_name ON activity_logs(app_name);
CREATE INDEX IF NOT EXISTS idx_activity_logs_phash ON activity_logs(phash);

-- 向量嵌入表
CREATE TABLE IF NOT EXISTS vector_embeddings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL,
    embedding BLOB NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (activity_id) REFERENCES activity_logs(id) ON DELETE CASCADE
);

-- 知识图谱节点表
CREATE TABLE IF NOT EXISTS knowledge_nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    node_group TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- 知识图谱边表
CREATE TABLE IF NOT EXISTS knowledge_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    target TEXT NOT NULL,
    value INTEGER DEFAULT 1,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (source) REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
);

-- 代理执行记录表
CREATE TABLE IF NOT EXISTS agent_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id INTEGER,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- 应用黑名单表
CREATE TABLE IF NOT EXISTS app_blocklist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL UNIQUE,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- 全文检索表（FTS5）
CREATE VIRTUAL TABLE IF NOT EXISTS activity_logs_fts USING fts5(
    ocr_text,
    content='activity_logs',
    content_rowid='id'
);

-- 触发器：同步 FTS5 索引
CREATE TRIGGER IF NOT EXISTS activity_logs_fts_insert AFTER INSERT ON activity_logs BEGIN
    INSERT INTO activity_logs_fts(rowid, ocr_text) VALUES (new.id, new.ocr_text);
END;

CREATE TRIGGER IF NOT EXISTS activity_logs_fts_update AFTER UPDATE ON activity_logs BEGIN
    DELETE FROM activity_logs_fts WHERE rowid = old.id;
    INSERT INTO activity_logs_fts(rowid, ocr_text) VALUES (new.id, new.ocr_text);
END;

CREATE TRIGGER IF NOT EXISTS activity_logs_fts_delete AFTER DELETE ON activity_logs BEGIN
    DELETE FROM activity_logs_fts WHERE rowid = old.id;
END;

