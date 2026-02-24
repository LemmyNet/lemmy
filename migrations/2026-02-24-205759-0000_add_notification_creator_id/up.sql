-- Add a nullable creator_id column to notifications, for private messages (also comments and posts)
-- Leave it nullable because mod actions shouldn't have a creator
ALTER TABLE notification
    ADD COLUMN creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- Update the data
-- Private messages
UPDATE
    notification n
SET
    creator_id = p.creator_id
FROM
    private_message p
WHERE
    n.private_message_id = p.id;

-- Posts
UPDATE
    notification n
SET
    creator_id = p.creator_id
FROM
    post p
WHERE
    n.post_id = p.id;

-- Comments
UPDATE
    notification n
SET
    creator_id = c.creator_id
FROM
    comment c
WHERE
    n.comment_id = c.id;

