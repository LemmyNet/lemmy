-- Drop the uniques
ALTER TABLE private_message
    DROP CONSTRAINT idx_private_message_ap_id;

ALTER TABLE post
    DROP CONSTRAINT idx_post_ap_id;

ALTER TABLE comment
    DROP CONSTRAINT idx_comment_ap_id;

ALTER TABLE user_
    DROP CONSTRAINT idx_user_actor_id;

ALTER TABLE community
    DROP CONSTRAINT idx_community_actor_id;

ALTER TABLE private_message
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE private_message
    ALTER COLUMN ap_id SET DEFAULT 'http://fake.com';

ALTER TABLE post
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE post
    ALTER COLUMN ap_id SET DEFAULT 'http://fake.com';

ALTER TABLE comment
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE comment
    ALTER COLUMN ap_id SET DEFAULT 'http://fake.com';

UPDATE
    private_message
SET
    ap_id = 'http://fake.com'
WHERE
    ap_id LIKE 'changeme_%';

UPDATE
    post
SET
    ap_id = 'http://fake.com'
WHERE
    ap_id LIKE 'changeme_%';

UPDATE
    comment
SET
    ap_id = 'http://fake.com'
WHERE
    ap_id LIKE 'changeme_%';

