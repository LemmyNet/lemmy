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
    -- this could reference local_user as notifications cannot be sent to remote users,
    -- but existing data all uses person.
    recipient_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    kind notification_type_enum NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    published_at timestamptz NOT NULL DEFAULT now()
);

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

-- copy data from comment_reply table
INSERT INTO notification (comment_id, recipient_id, kind, read, published_at)
SELECT
    comment_id,
    recipient_id,
    'Reply',
    read,
    published_at
FROM
    comment_reply;

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

