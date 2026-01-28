-- Fix FTS table: Convert from content-sync mode to standalone mode
-- FTS5 with content-sync mode may cause issues with concurrent writes

-- Step 1: Drop triggers first (required before dropping content-sync table)
DROP TRIGGER IF EXISTS activity_logs_fts_insert;
DROP TRIGGER IF EXISTS activity_logs_fts_update;
DROP TRIGGER IF EXISTS activity_logs_fts_delete;

-- Step 2: Drop the existing FTS table
-- Note: After dropping triggers, DROP TABLE should work even for content-sync tables
DROP TABLE IF EXISTS activity_logs_fts;

-- Step 3: Create a standalone FTS5 table (not content-synced)
-- This avoids the complex trigger-based synchronization
-- Using CREATE (not CREATE IF NOT EXISTS) to ensure table is recreated
CREATE VIRTUAL TABLE activity_logs_fts USING fts5(
    ocr_text,
    tokenize='unicode61'
);
