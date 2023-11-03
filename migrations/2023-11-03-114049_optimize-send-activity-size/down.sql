-- We can't restore ap_id values from hash, so drop all existing rows and restore schema.
DELETE FROM received_activity;

ALTER TABLE received_activity
    DROP COLUMN ap_id_hash;

ALTER TABLE received_activity
    ADD COLUMN id serial;

ALTER TABLE received_activity
    ADD COLUMN ap_id text NOT NULL;

ALTER TABLE received_activity
    ADD PRIMARY KEY (ap_id);

