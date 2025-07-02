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
    published_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id) = 1);

CREATE TABLE person_notification (
    notification_id int REFERENCES notification (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    -- this could reference local_user as notifications cannot be sent to remote users,
    -- but existing data all uses person.
    recipient_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    kind notification_type_enum NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    PRIMARY KEY (recipient_id, notification_id)
);

-- copy data from person_post_mention table
INSERT INTO notification (post_id, published_at)
SELECT
    post_id,
    published_at
FROM
    person_post_mention;

INSERT INTO person_notification (notification_id, recipient_id, kind, read)
SELECT
    n.id,
    recipient_id,
    'Mention',
    read
FROM
    person_post_mention m
    INNER JOIN notification n ON n.post_id = m.post_id;

-- copy data from person_comment_mention table
INSERT INTO notification (comment_id, published_at)
SELECT
    comment_id,
    published_at
FROM
    person_comment_mention;

INSERT INTO person_notification (notification_id, recipient_id, kind, read)
SELECT
    n.id,
    recipient_id,
    'Mention',
    read
FROM
    person_comment_mention m
    INNER JOIN notification n ON n.comment_id = m.comment_id;

-- copy data from comment_reply table
INSERT INTO notification (comment_id, published_at)
SELECT
    comment_id,
    published_at
FROM
    comment_reply;

INSERT INTO person_notification (notification_id, recipient_id, kind, read)
SELECT
    n.id,
    recipient_id,
    'Reply',
    read
FROM
    comment_reply m
    INNER JOIN notification n ON n.comment_id = m.comment_id;

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

