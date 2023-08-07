-- Following this issue : https://github.com/LemmyNet/lemmy/issues/957
-- Creating a unique changeme actor_id
CREATE OR REPLACE FUNCTION generate_unique_changeme ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'changeme_' || string_agg(substr('abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789', ceil(random() * 62)::integer, 1), '')
    FROM
        generate_series(1, 20)
$$;

-- Need to delete the possible community and user dupes for ones that don't start with the fake one
-- A few test inserts, to make sure this removes later dupes
-- insert into community (name, title, category_id, creator_id) values ('testcom', 'another testcom', 1, 2);
DELETE FROM community a USING (
    SELECT
        min(id) AS id,
        actor_id
    FROM
        community
    GROUP BY
        actor_id
    HAVING
        count(*) > 1) b
WHERE
    a.actor_id = b.actor_id
    AND a.id <> b.id;

DELETE FROM user_ a USING (
    SELECT
        min(id) AS id,
        actor_id
    FROM
        user_
    GROUP BY
        actor_id
    HAVING
        count(*) > 1) b
WHERE
    a.actor_id = b.actor_id
    AND a.id <> b.id;

-- Replacing the current default on the columns, to the unique one
UPDATE
    community
SET
    actor_id = generate_unique_changeme ()
WHERE
    actor_id = 'http://fake.com';

UPDATE
    user_
SET
    actor_id = generate_unique_changeme ()
WHERE
    actor_id = 'http://fake.com';

-- Add the unique indexes
ALTER TABLE community
    ALTER COLUMN actor_id SET NOT NULL;

ALTER TABLE community
    ALTER COLUMN actor_id SET DEFAULT generate_unique_changeme ();

ALTER TABLE user_
    ALTER COLUMN actor_id SET NOT NULL;

ALTER TABLE user_
    ALTER COLUMN actor_id SET DEFAULT generate_unique_changeme ();

-- Add lowercase uniqueness too
DROP INDEX idx_user_name_lower_actor_id;

CREATE UNIQUE INDEX idx_user_lower_actor_id ON user_ (lower(actor_id));

CREATE UNIQUE INDEX idx_community_lower_actor_id ON community (lower(actor_id));

