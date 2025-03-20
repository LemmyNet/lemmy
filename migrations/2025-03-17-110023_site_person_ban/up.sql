ALTER TABLE instance_actions
    ADD COLUMN received_ban timestamptz;

ALTER TABLE instance_actions
    ADD COLUMN ban_expires timestamptz;

ALTER TABLE mod_ban
    ADD COLUMN instance_id int NOT NULL REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

-- insert existing bans into instance_actions table, assuming they were all banned from home instance
INSERT INTO instance_actions (person_id, instance_id, received_ban, ban_expires)
SELECT
    id,
    instance_id,
    now(),
    ban_expires
FROM
    person
WHERE
    banned;

ALTER TABLE person
    DROP COLUMN banned;

ALTER TABLE person
    DROP COLUMN ban_expires;

