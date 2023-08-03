-- Add unique ap_id for private_message, comment, and post
-- Need to delete the possible dupes for ones that don't start with the fake one
DELETE FROM private_message a USING (
    SELECT
        min(id) AS id,
        ap_id
    FROM
        private_message
    GROUP BY
        ap_id
    HAVING
        count(*) > 1) b
WHERE
    a.ap_id = b.ap_id
    AND a.id <> b.id;

DELETE FROM post a USING (
    SELECT
        min(id) AS id,
        ap_id
    FROM
        post
    GROUP BY
        ap_id
    HAVING
        count(*) > 1) b
WHERE
    a.ap_id = b.ap_id
    AND a.id <> b.id;

DELETE FROM comment a USING (
    SELECT
        min(id) AS id,
        ap_id
    FROM
        comment
    GROUP BY
        ap_id
    HAVING
        count(*) > 1) b
WHERE
    a.ap_id = b.ap_id
    AND a.id <> b.id;

-- Replacing the current default on the columns, to the unique one
UPDATE
    private_message
SET
    ap_id = generate_unique_changeme ()
WHERE
    ap_id = 'http://fake.com';

UPDATE
    post
SET
    ap_id = generate_unique_changeme ()
WHERE
    ap_id = 'http://fake.com';

UPDATE
    comment
SET
    ap_id = generate_unique_changeme ()
WHERE
    ap_id = 'http://fake.com';

-- Add the unique indexes
ALTER TABLE private_message
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE private_message
    ALTER COLUMN ap_id SET DEFAULT generate_unique_changeme ();

ALTER TABLE post
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE post
    ALTER COLUMN ap_id SET DEFAULT generate_unique_changeme ();

ALTER TABLE comment
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE comment
    ALTER COLUMN ap_id SET DEFAULT generate_unique_changeme ();

-- Add the uniques, for user_ and community too
ALTER TABLE private_message
    ADD CONSTRAINT idx_private_message_ap_id UNIQUE (ap_id);

ALTER TABLE post
    ADD CONSTRAINT idx_post_ap_id UNIQUE (ap_id);

ALTER TABLE comment
    ADD CONSTRAINT idx_comment_ap_id UNIQUE (ap_id);

ALTER TABLE user_
    ADD CONSTRAINT idx_user_actor_id UNIQUE (actor_id);

ALTER TABLE community
    ADD CONSTRAINT idx_community_actor_id UNIQUE (actor_id);

