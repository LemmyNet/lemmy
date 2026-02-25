ALTER TABLE modlog
    ADD COLUMN bulk_action_parent_id int REFERENCES modlog (id) ON DELETE SET NULL;

CREATE INDEX idx_modlog_bulk_action_parent_id ON modlog (bulk_action_parent_id);

