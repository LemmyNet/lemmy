ALTER TABLE instance_actions
    ADD COLUMN received_ban timestamptz;

ALTER TABLE instance_actions
    ADD COLUMN ban_expires timestamptz;

ALTER TABLE mod_ban
    ADD COLUMN instance_id int NOT NULL REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE person
    DROP COLUMN banned;

ALTER TABLE person
    DROP COLUMN ban_expires;

-- TODO: insert bans into instance_actions table