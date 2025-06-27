CREATE TYPE post_or_comment_notification_type_enum AS enum (
    'mention',
    'parent_reply',
    'subscribed'
);

CREATE TABLE post_or_comment_notification (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE,
    read bool NOT NULL DEFAULT FALSE,
    kind post_or_comment_notification_type_enum NOT NULL,
    published_at timestamptz NOT NULL DEFAULT now()
);

-- TODO: transfer data from existing tables
DELETE FROM inbox_combined;

ALTER TABLE inbox_combined
    DROP CONSTRAINT inbox_combined_check;

ALTER TABLE inbox_combined
    DROP COLUMN comment_reply_id,
    DROP COLUMN person_comment_mention_id,
    DROP COLUMN person_post_mention_id,
    ADD COLUMN post_or_comment_notification_id int REFERENCES post_or_comment_notification (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE;

ALTER TABLE inbox_combined
    ADD CONSTRAINT inbox_combined_check CHECK (num_nonnulls (post_or_comment_notification_id, private_message_id) = 1);

DROP TABLE person_post_mention, person_comment_mention, comment_reply;

