ALTER TABLE instance_actions
    ADD COLUMN received_ban timestamptz;

ALTER TABLE instance_actions
    ADD COLUMN ban_expires timestamptz;

-- TODO: could be not null
ALTER TABLE mod_ban
    ADD COLUMN instance_id int NOT NULL REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

alter table person drop column banned;alter table person drop column ban_expires;