DROP INDEX IF EXISTS idx_modlog_bulk_action_parent_id;

ALTER TABLE modlog
    DROP COLUMN bulk_action_parent_id;

