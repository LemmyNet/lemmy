-- Delete any preexisting notification duplicates
DELETE FROM notification n USING notification dupe
WHERE n.comment_id IS NOT NULL
    AND n.modlog_id IS NULL
    AND n.comment_id = dupe.comment_id
    AND n.recipient_id = dupe.recipient_id
    AND n.kind = dupe.kind
    AND n.id > dupe.id;

DELETE FROM notification n USING notification dupe
WHERE n.post_id IS NOT NULL
    AND n.comment_id IS NULL
    AND n.modlog_id IS NULL
    AND n.post_id = dupe.post_id
    AND n.recipient_id = dupe.recipient_id
    AND n.kind = dupe.kind
    AND n.id > dupe.id;

DELETE FROM notification n USING notification dupe
WHERE n.private_message_id IS NOT NULL
    AND n.private_message_id = dupe.private_message_id
    AND n.recipient_id = dupe.recipient_id
    AND n.id > dupe.id;

DELETE FROM notification n USING notification dupe
WHERE n.modlog_id IS NOT NULL
    AND n.modlog_id = dupe.modlog_id
    AND n.recipient_id = dupe.recipient_id
    AND n.id > dupe.id;

-- Create unique indexes for notifications
CREATE UNIQUE INDEX idx_notification_comment_recipient_kind ON notification (comment_id, recipient_id, kind)
WHERE
    comment_id IS NOT NULL AND modlog_id IS NULL;

CREATE UNIQUE INDEX idx_notification_post_recipient_kind ON notification (post_id, recipient_id, kind)
WHERE
    post_id IS NOT NULL AND comment_id IS NULL AND modlog_id IS NULL;

CREATE UNIQUE INDEX idx_notification_private_message_recipient ON notification (private_message_id, recipient_id)
WHERE
    private_message_id IS NOT NULL;

CREATE UNIQUE INDEX idx_notification_modlog_recipient ON notification (modlog_id, recipient_id)
WHERE
    modlog_id IS NOT NULL;

