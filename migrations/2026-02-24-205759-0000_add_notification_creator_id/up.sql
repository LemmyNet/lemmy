-- Add a creator_id column to notifications.
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

-- Mod actions
UPDATE
    notification n
SET
    creator_id = m.mod_id
FROM
    modlog m
WHERE
    n.modlog_id = m.id;

-- Make column not null
ALTER TABLE notification
    ALTER COLUMN creator_id SET NOT NULL;

-- Create an index
CREATE INDEX idx_notification_creator ON notification (creator_id);

