ALTER TABLE instance_actions
    ADD COLUMN received_ban timestamptz;

ALTER TABLE instance_actions
    ADD COLUMN ban_expires timestamptz;

ALTER TABLE mod_ban
    ADD COLUMN instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

UPDATE
    mod_ban
SET
    instance_id = person.instance_id
FROM
    person
WHERE
    mod_ban.instance_id IS NULL
    AND mod_ban.mod_person_id = person.id;

ALTER TABLE mod_ban
    ALTER COLUMN instance_id SET NOT NULL;

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

