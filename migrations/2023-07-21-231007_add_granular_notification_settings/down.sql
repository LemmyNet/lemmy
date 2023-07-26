ALTER TABLE local_user
    DROP COLUMN send_notifications_for_post_replies;

ALTER TABLE local_user
    DROP COLUMN send_notifications_for_comment_replies;

ALTER TABLE local_user
    DROP COLUMN send_notifications_for_private_messages;

ALTER TABLE local_user
    DROP COLUMN send_notifications_for_mentions;

