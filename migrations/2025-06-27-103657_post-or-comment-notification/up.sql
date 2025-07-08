-- create new data types
CREATE TYPE notification_type_enum AS enum (
    'Mention',
    'Reply',
    'Subscribed',
    'PrivateMessage'
);

CREATE TABLE notification (
    id serial PRIMARY KEY,
    post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE,
    private_message_id int REFERENCES private_message (id) ON UPDATE CASCADE ON DELETE CASCADE,
    recipient_id int REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    kind notification_type_enum NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    published_at timestamptz NOT NULL DEFAULT now()
);

-- copy data from person_post_mention table
INSERT INTO notification (post_id, recipient_id, kind, read, published_at)
SELECT
    post_id,
    local_user.id,
    'Mention',
    read,
    published_at
FROM
    person_post_mention
    LEFT JOIN local_user ON recipient_id = local_user.person_id;

-- copy data from person_comment_mention table
INSERT INTO notification (comment_id, recipient_id, kind, read, published_at)
SELECT
    comment_id,
    local_user.id,
    'Mention',
    read,
    published_at
FROM
    person_comment_mention
    LEFT JOIN local_user ON recipient_id = local_user.person_id;

-- copy data from comment_reply table
INSERT INTO notification (comment_id, recipient_id, kind, read, published_at)
SELECT
    comment_id,
    local_user.id,
    'Reply',
    read,
    published_at
FROM
    comment_reply
    LEFT JOIN local_user ON recipient_id = local_user.person_id;

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id) = 1);

CREATE INDEX idx_notification_recipient_published ON notification (recipient_id, published_at);

DROP TABLE inbox_combined, person_post_mention, person_comment_mention, comment_reply;

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

