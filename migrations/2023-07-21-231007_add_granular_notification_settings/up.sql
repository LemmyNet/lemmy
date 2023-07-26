ALTER TABLE local_user
    ADD COLUMN send_notifications_for_post_replies boolean DEFAULT TRUE NOT NULL;

ALTER TABLE local_user
    ADD COLUMN send_notifications_for_comment_replies boolean DEFAULT TRUE NOT NULL;

ALTER TABLE local_user
    ADD COLUMN send_notifications_for_private_messages boolean DEFAULT TRUE NOT NULL;

ALTER TABLE local_user
    ADD COLUMN send_notifications_for_mentions boolean DEFAULT TRUE NOT NULL;

