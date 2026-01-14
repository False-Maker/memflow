-- 智能代理自动化提案与审计
-- 迁移文件：0006_agent_automation.sql

-- ============================================
-- 自动化提案表（持久化提案，供审计引用）
-- ============================================
CREATE TABLE IF NOT EXISTS automation_proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    confidence REAL NOT NULL,
    risk_level TEXT NOT NULL, -- low | medium | high
    steps_json TEXT NOT NULL, -- JSON 数组（步骤）
    evidence_json TEXT,       -- JSON 数组（证据，可选）
    created_at INTEGER DEFAULT (strftime('%s','now'))
);

CREATE INDEX IF NOT EXISTS idx_automation_proposals_created
    ON automation_proposals(created_at DESC);

-- ============================================
-- 扩展 agent_executions（审计信息增强）
-- ============================================
ALTER TABLE agent_executions ADD COLUMN finished_at INTEGER;
ALTER TABLE agent_executions ADD COLUMN error_message TEXT;
ALTER TABLE agent_executions ADD COLUMN metadata_json TEXT;

CREATE INDEX IF NOT EXISTS idx_agent_executions_created
    ON agent_executions(created_at DESC);


