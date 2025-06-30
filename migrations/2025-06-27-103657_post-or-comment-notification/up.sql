CREATE TYPE notification_type_enum AS enum (
    'Mention',
    'Reply',
    'Subscribed',
    'PrivateMessage'
);

CREATE TABLE notification (
    id serial PRIMARY KEY,
    -- TODO: local user id would make sense here as we can only notify local users
    post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id) = 1);

create table local_user_notification (
    notification_id int REFERENCES notification (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    recipient_id int REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    kind notification_type_enum NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    primary key(recipient_id,notification_id)
);

-- TODO: transfer data from existing tables
DROP TABLE person_post_mention, person_comment_mention, comment_reply;

