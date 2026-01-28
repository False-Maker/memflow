-- 恢复 FTS5 触发器，确保新数据自动同步到全文检索索引
-- 由于 migration 0003 创建的是独立 FTS 表（非 content-sync），触发器需要手动管理数据同步

-- 插入触发器：当新活动记录插入且有 OCR 文本时，同步到 FTS
CREATE TRIGGER IF NOT EXISTS activity_logs_fts_insert 
AFTER INSERT ON activity_logs 
WHEN NEW.ocr_text IS NOT NULL
BEGIN
    INSERT INTO activity_logs_fts(rowid, ocr_text) VALUES (NEW.id, NEW.ocr_text);
END;

-- 更新触发器：当 OCR 文本更新时，同步到 FTS
CREATE TRIGGER IF NOT EXISTS activity_logs_fts_update 
AFTER UPDATE OF ocr_text ON activity_logs 
BEGIN
    -- 先删除旧记录（如果存在）
    DELETE FROM activity_logs_fts WHERE rowid = OLD.id;
    -- 如果新值不为空，插入新记录
    INSERT INTO activity_logs_fts(rowid, ocr_text) 
    SELECT NEW.id, NEW.ocr_text WHERE NEW.ocr_text IS NOT NULL;
END;

-- 删除触发器：当活动记录删除时，同步删除 FTS 索引
CREATE TRIGGER IF NOT EXISTS activity_logs_fts_delete 
AFTER DELETE ON activity_logs 
BEGIN
    DELETE FROM activity_logs_fts WHERE rowid = OLD.id;
END;

