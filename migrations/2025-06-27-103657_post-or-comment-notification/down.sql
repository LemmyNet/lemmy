CREATE TABLE person_post_mention (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read bool NOT NULL DEFAULT FALSE,
    published_at timestamptz DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.person_post_mention
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

ALTER TABLE inbox_combined
    DROP CONSTRAINT inbox_combined_check;

ALTER TABLE inbox_combined
    ADD COLUMN comment_reply_id int REFERENCES comment_reply (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE,
    ADD COLUMN person_comment_mention_id int REFERENCES person_comment_mention (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE,
    ADD COLUMN person_post_mention_id int REFERENCES person_post_mention (id) ON UPDATE CASCADE ON DELETE CASCADE UNIQUE,
    DROP COLUMN notification_id;

ALTER TABLE inbox_combined
    ADD CONSTRAINT inbox_combined_check CHECK (num_nonnulls (comment_reply_id, person_comment_mention_id, person_post_mention_id, private_message_id) = 1);

CREATE INDEX idx_comment_reply_comment ON comment_reply USING btree (comment_id);

CREATE INDEX idx_comment_reply_recipient ON public.comment_reply USING btree (recipient_id);

CREATE INDEX idx_comment_reply_published ON public.comment_reply USING btree (published_at DESC);

DROP TABLE notification;

DROP TYPE notification_type_enum;

