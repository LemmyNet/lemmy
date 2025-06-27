-- Creates combined tables for
-- person_content: (comment, post)
-- person_saved: (comment, post)
CREATE TABLE person_content_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

CREATE INDEX idx_person_content_combined_published ON person_content_combined (published DESC, id DESC);

-- Updating the history
INSERT INTO person_content_combined (published, post_id)
SELECT
    published,
    id
FROM
    post
WHERE
    published > now() - interval '1 month'
ON CONFLICT (post_id)
    DO UPDATE SET
        published = excluded.published;

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_id)
SELECT
    'post',
    'person_content_combined',
    min(id)
FROM
    post
WHERE
    published > now() - interval '1 month';

INSERT INTO person_content_combined (published, comment_id)
SELECT
    published,
    id
FROM
    comment
WHERE
    published > now() - interval '1 month'
ON CONFLICT (comment_id)
    DO UPDATE SET
        published = excluded.published;

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_id)
SELECT
    'comment',
    'person_content_combined',
    min(id)
FROM
    comment
WHERE
    published > now() - interval '1 month';

-- This one is special, because you use the saved date, not the ordinary published
CREATE TABLE person_saved_combined (
    id serial PRIMARY KEY,
    saved timestamptz NOT NULL,
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    UNIQUE (person_id, post_id),
    UNIQUE (person_id, comment_id)
);

-- Updating the history
-- This is for local_users only
INSERT INTO person_saved_combined (saved, person_id, post_id)
SELECT
    pa.saved,
    pa.person_id,
    pa.post_id
FROM
    post_actions pa,
    local_user lu
WHERE
    pa.person_id = lu.person_id
    AND pa.saved IS NOT NULL
    AND pa.saved > now() - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        saved = excluded.saved;

-- Leave the last_scanned_ids blank, since post_actions might not be filled yet.
-- You need the post_actions table to be filled entirely before filling this history.
INSERT INTO history_status (source, dest)
    VALUES ('post_actions', 'person_saved_combined');

INSERT INTO person_saved_combined (saved, person_id, comment_id)
SELECT
    ca.saved,
    ca.person_id,
    ca.comment_id
FROM
    comment_actions ca,
    local_user lu
WHERE
    ca.person_id = lu.person_id
    AND ca.saved IS NOT NULL
    AND ca.saved > now() - interval '1 month'
ON CONFLICT (person_id,
    comment_id)
    DO UPDATE SET
        saved = excluded.saved;

INSERT INTO history_status (source, dest)
    VALUES ('comment_actions', 'person_saved_combined');

CREATE INDEX idx_person_saved_combined_published ON person_saved_combined (saved DESC, id DESC);

CREATE INDEX idx_person_saved_combined ON person_saved_combined (person_id);

ALTER TABLE person_saved_combined
    ALTER CONSTRAINT person_saved_combined_person_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_saved_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_saved_combined_comment_id_fkey NOT DEFERRABLE,
    ADD CONSTRAINT person_saved_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

ALTER TABLE person_content_combined
    ALTER CONSTRAINT person_content_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_content_combined_comment_id_fkey NOT DEFERRABLE,
    ADD CONSTRAINT person_content_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

