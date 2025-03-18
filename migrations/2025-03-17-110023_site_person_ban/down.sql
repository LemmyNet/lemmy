ALTER TABLE instance_actions
    DROP COLUMN received_ban;

ALTER TABLE instance_actions
    DROP COLUMN ban_expires;

ALTER TABLE mod_ban
    DROP COLUMN instance_id;

