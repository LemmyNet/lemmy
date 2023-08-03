ALTER TABLE activity
    ALTER COLUMN ap_id DROP NOT NULL;

CREATE UNIQUE INDEX idx_activity_unique_apid ON activity ((data ->> 'id'::text));

DROP INDEX idx_activity_ap_id;

