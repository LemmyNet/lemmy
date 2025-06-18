-- Creates combined tables for
-- Inbox: (replies, comment mentions, post mentions, and private_messages)
-- Also add post mentions, since these didn't exist before.
-- Rename the person_mention table to person_comment_mention
ALTER TABLE person_mention RENAME TO person_comment_mention;

-- Create the new post_mention table
CREATE TABLE person_post_mention (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    read boolean DEFAULT FALSE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    UNIQUE (recipient_id, post_id)
);

CREATE TABLE inbox_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    comment_reply_id int UNIQUE REFERENCES comment_reply ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    person_comment_mention_id int UNIQUE REFERENCES person_comment_mention ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    person_post_mention_id int UNIQUE REFERENCES person_post_mention ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    private_message_id int UNIQUE REFERENCES private_message ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

CREATE INDEX idx_inbox_combined_published ON inbox_combined (published DESC, id DESC);

CREATE INDEX idx_inbox_combined_published_asc ON inbox_combined (reverse_timestamp_sort (published) DESC, id DESC);

-- Updating the history
INSERT INTO inbox_combined (published, comment_reply_id, person_comment_mention_id, person_post_mention_id, private_message_id)
SELECT
    published,
    id,
    NULL::int,
    NULL::int,
    NULL::int
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

ALTER TABLE person_post_mention
    ALTER CONSTRAINT person_post_mention_recipient_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_post_mention_post_id_fkey NOT DEFERRABLE;

-- Make sure only one of the columns is not null
ALTER TABLE inbox_combined
    ADD CONSTRAINT inbox_combined_check CHECK (num_nonnulls (comment_reply_id, person_comment_mention_id, person_post_mention_id, private_message_id) = 1),
    ALTER CONSTRAINT inbox_combined_comment_reply_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT inbox_combined_person_comment_mention_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT inbox_combined_person_post_mention_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT inbox_combined_private_message_id_fkey NOT DEFERRABLE;

