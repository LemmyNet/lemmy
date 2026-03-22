-- This adds (sometimes redundant) id columns to source tables, so that views can be built without needing to cascade joins to multiple tables.
-- Add a redundant community_id back to the comment table
ALTER TABLE comment
    ADD COLUMN community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Fill the rows
UPDATE
    comment AS c
SET
    community_id = p.community_id
FROM
    post AS p
WHERE
    c.post_id = p.id;

-- Set it to not null
ALTER TABLE comment
    ALTER COLUMN community_id SET NOT NULL;

CREATE INDEX idx_comment_community ON comment (community_id);

-- Notification
-- Add an optional instance_id (for modlog view)
ALTER TABLE notification
    ADD COLUMN instance_id int REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    -- drop the check constraint
    DROP CONSTRAINT notification_check;

-- Currently the notification _id columns are only referenced by that item (ie a comment_id and not a post_id).
-- Change this so that the post_id column is also filled for comments.
UPDATE
    notification AS n
SET
    post_id = c.post_id,
    community_id = c.community_id
FROM
    comment AS c
WHERE
    n.comment_id = c.id;

-- Update post community id
UPDATE
    notification AS n
SET
    community_id = p.community_id
FROM
    post AS p
WHERE
    n.post_id = p.id;

-- Update all the modlog-related columns also
UPDATE
    notification AS n
SET
    post_id = m.target_post_id,
    comment_id = m.target_comment_id,
    community_id = m.target_community_id,
    instance_id = m.target_instance_id
    -- person_id is already handled by recipient id
FROM
    modlog AS m
WHERE
    n.modlog_id = m.id;

CREATE INDEX idx_notification_instance ON notification (instance_id);

CREATE INDEX idx_notification_community ON notification (community_id);

-- Report combined needs all the items added, item_creator, report_creator (required), resolver, post, comment, community, private message
ALTER TABLE report_combined
    ADD COLUMN item_creator_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN report_creator_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN resolver_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN comment_id int REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN private_message_id int REFERENCES private_message (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Comment report
UPDATE
    report_combined AS rc
SET
    item_creator_id = c.creator_id,
    report_creator_id = cr.creator_id,
    resolver_id = cr.resolver_id,
    comment_id = c.id,
    post_id = c.post_id,
    community_id = c.community_id
FROM
    comment_report AS cr,
    comment AS c
WHERE
    rc.comment_report_id = cr.id
    AND cr.comment_id = c.id;

-- Post report
UPDATE
    report_combined AS rc
SET
    item_creator_id = p.creator_id,
    report_creator_id = pr.creator_id,
    resolver_id = pr.resolver_id,
    post_id = p.id,
    community_id = p.community_id
FROM
    post_report AS pr,
    post AS p
WHERE
    rc.post_report_id = pr.id
    AND pr.post_id = p.id;

-- Community report
UPDATE
    report_combined AS rc
SET
    report_creator_id = cr.creator_id,
    resolver_id = cr.resolver_id,
    community_id = cr.community_id
FROM
    community_report AS cr
WHERE
    rc.community_report_id = cr.id;

-- Private message report
UPDATE
    report_combined AS rc
SET
    item_creator_id = p.creator_id,
    report_creator_id = pr.creator_id,
    resolver_id = pr.resolver_id,
    private_message_id = pr.private_message_id
FROM
    private_message_report AS pr,
    private_message p
WHERE
    rc.community_report_id = pr.id
    AND pr.private_message_id = p.id;

ALTER TABLE report_combined
    ALTER COLUMN report_creator_id SET NOT NULL;

CREATE INDEX idx_report_combined_item_creator ON report_combined (item_creator_id);

CREATE INDEX idx_report_combined_report_creator ON report_combined (report_creator_id);

CREATE INDEX idx_report_combined_resolver ON report_combined (resolver_id);

CREATE INDEX idx_report_combined_post ON report_combined (post_id);

CREATE INDEX idx_report_combined_comment ON report_combined (comment_id);

CREATE INDEX idx_report_combined_community ON report_combined (community_id);

CREATE INDEX idx_report_combined_private_message ON report_combined (private_message_id);

-- Add the post_id for person_saved, person_liked, and person_content combined comments
-- drop the check and unique constraint
ALTER TABLE person_saved_combined
    ADD COLUMN community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    DROP CONSTRAINT person_saved_combined_check,
    DROP CONSTRAINT person_saved_combined_person_id_comment_id_key,
    DROP CONSTRAINT person_saved_combined_person_id_post_id_key;

UPDATE
    person_saved_combined AS psc
SET
    community_id = p.community_id
FROM
    post p
WHERE
    psc.post_id = p.id;

UPDATE
    person_saved_combined AS psc
SET
    post_id = c.post_id,
    community_id = c.community_id
FROM
    comment c
WHERE
    psc.comment_id = c.id;

-- Set it to not null
ALTER TABLE person_saved_combined
    ALTER COLUMN post_id SET NOT NULL,
    ALTER COLUMN community_id SET NOT NULL;

CREATE INDEX idx_person_saved_combined_community ON person_saved_combined (community_id);

-- drop the check constraint
ALTER TABLE person_liked_combined
    ADD COLUMN community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    DROP CONSTRAINT person_liked_combined_check,
    DROP CONSTRAINT person_liked_combined_person_id_comment_id_key,
    DROP CONSTRAINT person_liked_combined_person_id_post_id_key,
    ADD CONSTRAINT person_liked_combined_unique UNIQUE nulls NOT DISTINCT (person_id, comment_id, post_id);

UPDATE
    person_liked_combined AS psc
SET
    community_id = p.community_id
FROM
    post p
WHERE
    psc.post_id = p.id;

UPDATE
    person_liked_combined AS psc
SET
    post_id = c.post_id,
    community_id = c.community_id
FROM
    comment c
WHERE
    psc.comment_id = c.id;

-- Set it to not null
ALTER TABLE person_liked_combined
    ALTER COLUMN post_id SET NOT NULL,
    ALTER COLUMN community_id SET NOT NULL;

CREATE INDEX idx_person_liked_combined_community ON person_liked_combined (community_id);

-- drop the check constraint
ALTER TABLE person_content_combined
    ADD COLUMN community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    DROP CONSTRAINT person_content_combined_check,
    DROP CONSTRAINT person_content_combined_post_id_key,
    DROP CONSTRAINT person_content_combined_comment_id_key;

UPDATE
    person_content_combined AS psc
SET
    community_id = p.community_id
FROM
    post p
WHERE
    psc.post_id = p.id;

UPDATE
    person_content_combined AS psc
SET
    post_id = c.post_id,
    community_id = c.community_id
FROM
    comment c
WHERE
    psc.comment_id = c.id;

-- Set it to not null
ALTER TABLE person_content_combined
    ALTER COLUMN post_id SET NOT NULL,
    ALTER COLUMN community_id SET NOT NULL;

CREATE INDEX idx_person_content_combined_post ON person_content_combined (post_id);

CREATE INDEX idx_person_content_combined_comment ON person_content_combined (comment_id);

CREATE INDEX idx_person_content_combined_community ON person_content_combined (community_id);

-- Adding a faster index on notification
CREATE INDEX idx_notification_published_id_recipient ON notification (published_at DESC, id DESC, recipient_id);

