CREATE TABLE person_post_mention (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    published_at timestamptz DEFAULT now() NOT NULL
);

ALTER TABLE ONLY person_post_mention
    ADD CONSTRAINT person_post_mention_unique UNIQUE (recipient_id, post_id);

CREATE TABLE person_mention (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    published_at timestamptz DEFAULT now() NOT NULL,
    UNIQUE (recipient_id, comment_id)
);

ALTER TABLE person_mention RENAME TO person_comment_mention;

CREATE TABLE comment_reply (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    published_at timestamptz DEFAULT now() NOT NULL,
    UNIQUE (recipient_id, comment_id)
);

CREATE TABLE inbox_combined (
    id serial PRIMARY KEY,
    comment_reply_id int REFERENCES comment_reply (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE,
    person_comment_mention_id int REFERENCES person_comment_mention (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE,
    person_post_mention_id int REFERENCES person_post_mention (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE,
    private_message_id int REFERENCES private_message (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE,
    published_at timestamptz NOT NULL
);

ALTER TABLE inbox_combined
    ADD CONSTRAINT inbox_combined_check CHECK (num_nonnulls (comment_reply_id, person_comment_mention_id, person_post_mention_id, private_message_id) = 1);

CREATE INDEX idx_comment_reply_comment ON comment_reply USING btree (comment_id);

CREATE INDEX idx_comment_reply_recipient ON comment_reply USING btree (recipient_id);

CREATE INDEX idx_comment_reply_published ON comment_reply USING btree (published_at DESC);

CREATE INDEX idx_inbox_combined_published_asc ON inbox_combined USING btree (reverse_timestamp_sort (published_at) DESC, id DESC);

CREATE INDEX idx_inbox_combined_published ON inbox_combined USING btree (published_at DESC, id DESC);

DROP TABLE notification;

DROP TYPE notification_type_enum;

