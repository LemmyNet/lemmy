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
INSERT INTO person_content_combined (published, post_id, comment_id)
SELECT
    published,
    id,
    NULL::int
FROM
    post
UNION ALL
SELECT
    published,
    NULL::int,
    id
FROM
    comment;

-- This one is special, because you use the saved date, not the ordinary published
CREATE TABLE person_saved_combined (
    id serial PRIMARY KEY,
    saved timestamptz NOT NULL,
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

-- Updating the history
INSERT INTO person_saved_combined (saved, person_id, post_id, comment_id)
SELECT
    saved,
    person_id,
    post_id,
    NULL::int
FROM
    post_actions
WHERE
    saved IS NOT NULL
UNION ALL
SELECT
    saved,
    person_id,
    NULL::int,
    comment_id
FROM
    comment_actions
WHERE
    saved IS NOT NULL;

CREATE INDEX idx_person_saved_combined_published ON person_saved_combined (saved DESC, id DESC);

CREATE INDEX idx_person_saved_combined ON person_saved_combined (person_id);

ALTER TABLE person_saved_combined
    ALTER CONSTRAINT person_saved_combined_person_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_saved_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_saved_combined_comment_id_fkey NOT DEFERRABLE,
    ADD CONSTRAINT person_saved_combined_person_id_post_id_key UNIQUE (person_id, post_id),
    ADD CONSTRAINT person_saved_combined_person_id_comment_id_key UNIQUE (person_id, comment_id),
    ADD CONSTRAINT person_saved_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

ALTER TABLE person_content_combined
    ALTER CONSTRAINT person_content_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_content_combined_comment_id_fkey NOT DEFERRABLE,
    ADD CONSTRAINT person_content_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

