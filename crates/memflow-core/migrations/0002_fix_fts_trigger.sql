-- Fix FTS5 update trigger for content-sync tables
-- FTS5 external content tables do not support UPDATE - must use DELETE + INSERT

-- Drop the incorrect trigger
DROP TRIGGER IF EXISTS activity_logs_fts_update;

-- Create the corrected trigger
CREATE TRIGGER activity_logs_fts_update AFTER UPDATE ON activity_logs BEGIN
    DELETE FROM activity_logs_fts WHERE rowid = old.id;
    INSERT INTO activity_logs_fts(rowid, ocr_text) VALUES (new.id, new.ocr_text);
END;
