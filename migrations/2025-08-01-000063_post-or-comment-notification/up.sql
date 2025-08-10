-- create new data types
CREATE TYPE notification_type_enum AS enum (
    'Mention',
    'Reply',
    'Subscribed',
    'PrivateMessage'
);

-- create notification table by renaming comment_reply, to avoid copying lots of data around
ALTER TABLE comment_reply RENAME TO notification;

ALTER INDEX idx_comment_reply_comment RENAME TO idx_notification_comment;

ALTER INDEX idx_comment_reply_recipient RENAME TO idx_notification_recipient;

ALTER INDEX idx_comment_reply_published RENAME TO idx_notification_published;

ALTER SEQUENCE comment_reply_id_seq
    RENAME TO notification_id_seq;

ALTER TABLE notification RENAME CONSTRAINT comment_reply_comment_id_fkey TO notification_comment_id_fkey;

ALTER TABLE notification RENAME CONSTRAINT comment_reply_pkey TO notification_pkey;

ALTER TABLE notification
    DROP CONSTRAINT comment_reply_recipient_id_comment_id_key;

ALTER TABLE notification RENAME CONSTRAINT comment_reply_recipient_id_fkey TO notification_recipient_id_fkey;

ALTER TABLE notification
    ADD COLUMN kind notification_type_enum NOT NULL DEFAULT 'Reply',
    ALTER COLUMN comment_id DROP NOT NULL,
    ADD COLUMN post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN private_message_id int REFERENCES private_message (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE notification
    ALTER COLUMN kind DROP DEFAULT;

-- copy data from person_post_mention table
INSERT INTO notification (post_id, recipient_id, kind, read, published_at)
SELECT
    post_id,
    recipient_id,
    'Mention',
    read,
    published_at
FROM
    person_post_mention;

-- copy data from person_comment_mention table
INSERT INTO notification (comment_id, recipient_id, kind, read, published_at)
SELECT
    comment_id,
    recipient_id,
    'Mention',
    read,
    published_at
FROM
    person_comment_mention;

-- copy data from private_message table
INSERT INTO notification (private_message_id, recipient_id, kind, read, published_at)
SELECT
    id,
    recipient_id,
    'PrivateMessage',
    read,
    published_at
FROM
    private_message;

ALTER TABLE private_message
    DROP COLUMN read;

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id) = 1);

CREATE INDEX idx_notification_recipient_published ON notification (recipient_id, published_at);

CREATE INDEX idx_notification_post ON notification (post_id);

CREATE INDEX idx_notification_private_message ON notification (private_message_id);

DROP TABLE inbox_combined, person_post_mention, person_comment_mention;

CREATE TYPE post_notifications_mode_enum AS enum (
    'AllComments',
    'RepliesAndMentions',
    'Mute'
);

ALTER TABLE post_actions
    ADD COLUMN notifications post_notifications_mode_enum;

CREATE TYPE community_notifications_mode_enum AS enum (
    'AllPostsAndComments',
    'AllPosts',
    'RepliesAndMentions',
    'Mute'
);

ALTER TABLE community_actions
    ADD COLUMN notifications community_notifications_mode_enum;

