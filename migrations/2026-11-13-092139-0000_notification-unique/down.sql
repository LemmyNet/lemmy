-- Drop the unique notification indexes
DROP INDEX IF EXISTS idx_notification_modlog_recipient;

DROP INDEX IF EXISTS idx_notification_private_message_recipient;

DROP INDEX IF EXISTS idx_notification_post_recipient_kind;

DROP INDEX IF EXISTS idx_notification_comment_recipient_kind;

