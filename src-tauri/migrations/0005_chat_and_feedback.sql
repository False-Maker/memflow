-- 对话历史与反馈系统
-- 迁移文件：0005_chat_and_feedback.sql

-- ============================================
-- 对话会话表
-- ============================================
CREATE TABLE IF NOT EXISTS chat_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,               -- 对话标题（首条问题的前50字）
    created_at INTEGER NOT NULL,       -- 创建时间（Unix 时间戳 ms）
    updated_at INTEGER NOT NULL        -- 最后更新时间
);

CREATE INDEX IF NOT EXISTS idx_chat_sessions_updated 
    ON chat_sessions(updated_at DESC);

-- ============================================
-- 对话消息表
-- ============================================
CREATE TABLE IF NOT EXISTS chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,       -- 关联的会话 ID
    role TEXT NOT NULL,                -- 'user' | 'assistant'
    content TEXT NOT NULL,             -- 消息内容
    context_ids TEXT,                  -- 关联的活动 ID（JSON 数组，用于溯源）
    created_at INTEGER NOT NULL,       -- 消息时间戳
    FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_session 
    ON chat_messages(session_id);

-- ============================================
-- 消息评价表
-- ============================================
CREATE TABLE IF NOT EXISTS message_ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL UNIQUE, -- 关联的消息 ID（每条消息只能评价一次）
    rating INTEGER NOT NULL,            -- 1 = 点赞, -1 = 点踩
    comment TEXT,                       -- 可选的评价备注
    created_at INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES chat_messages(id) ON DELETE CASCADE
);

-- ============================================
-- 用户反馈表
-- ============================================
CREATE TABLE IF NOT EXISTS user_feedbacks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,             -- 'bug' | 'feature' | 'experience' | 'other'
    title TEXT NOT NULL,                -- 反馈标题
    content TEXT NOT NULL,              -- 详细描述
    screenshot_path TEXT,               -- 附加截图路径（可选）
    context_session_id INTEGER,         -- 关联的对话会话 ID（可选）
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending' | 'noted' | 'resolved'
    created_at INTEGER NOT NULL,
    FOREIGN KEY (context_session_id) REFERENCES chat_sessions(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_user_feedbacks_created 
    ON user_feedbacks(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_user_feedbacks_category 
    ON user_feedbacks(category);

-- ============================================
-- 对话内容全文检索表（FTS5）
-- ============================================
CREATE VIRTUAL TABLE IF NOT EXISTS chat_messages_fts USING fts5(
    content,
    content='chat_messages',
    content_rowid='id'
);

-- 触发器：同步 FTS5 索引
CREATE TRIGGER IF NOT EXISTS chat_messages_fts_insert AFTER INSERT ON chat_messages BEGIN
    INSERT INTO chat_messages_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER IF NOT EXISTS chat_messages_fts_update AFTER UPDATE ON chat_messages BEGIN
    DELETE FROM chat_messages_fts WHERE rowid = old.id;
    INSERT INTO chat_messages_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER IF NOT EXISTS chat_messages_fts_delete AFTER DELETE ON chat_messages BEGIN
    DELETE FROM chat_messages_fts WHERE rowid = old.id;
END;






