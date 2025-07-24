-- Creates combined tables for
-- Inbox: (replies, comment mentions, post mentions, and private_messages)
-- Also add post mentions, since these didn't exist before.
-- Rename the person_mention table to person_comment_mention
ALTER TABLE person_mention RENAME TO person_comment_mention;

-- Create the new post_mention table
CREATE TABLE person_post_mention (
    id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    recipient_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read boolean DEFAULT FALSE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    UNIQUE (recipient_id, post_id)
);

-- Updating the history
CREATE TABLE inbox_combined AS
SELECT
    published,
    id AS comment_reply_id,
    NULL::int AS person_comment_mention_id,
    NULL::int AS person_post_mention_id,
    NULL::int AS private_message_id
FROM
    comment_reply
UNION ALL
SELECT
    published,
    NULL::int,
    id,
    NULL::int,
    NULL::int
FROM
    person_comment_mention
UNION ALL
SELECT
    published,
    NULL::int,
    NULL::int,
    id,
    NULL::int
FROM
    person_post_mention
UNION ALL
SELECT
    published,
    NULL::int,
    NULL::int,
    NULL::int,
    id
FROM
    private_message;

ALTER TABLE inbox_combined
    ADD COLUMN id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    ALTER COLUMN published SET NOT NULL,
    ADD CONSTRAINT inbox_combined_comment_reply_id_fkey FOREIGN KEY (comment_reply_id) REFERENCES comment_reply ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT inbox_combined_person_comment_mention_id_fkey FOREIGN KEY (person_comment_mention_id) REFERENCES person_comment_mention ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT inbox_combined_person_post_mention_id_fkey FOREIGN KEY (person_post_mention_id) REFERENCES person_post_mention ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT inbox_combined_private_message_id_fkey FOREIGN KEY (private_message_id) REFERENCES private_message ON UPDATE CASCADE ON DELETE CASCADE,
    ADD UNIQUE (comment_reply_id),
    ADD UNIQUE (person_comment_mention_id),
    ADD UNIQUE (person_post_mention_id),
    ADD UNIQUE (private_message_id),
    ADD CONSTRAINT inbox_combined_check CHECK (num_nonnulls (comment_reply_id, person_comment_mention_id, person_post_mention_id, private_message_id) = 1);

CREATE INDEX idx_inbox_combined_published ON inbox_combined (published DESC, id DESC);

CREATE INDEX idx_inbox_combined_published_asc ON inbox_combined (reverse_timestamp_sort (published) DESC, id DESC);

