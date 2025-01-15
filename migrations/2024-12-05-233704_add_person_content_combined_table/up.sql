-- Creates combined tables for
-- person_content: (comment, post)
-- person_saved: (comment, post)
CREATE TABLE person_content_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (post_id, comment_id) = 1)
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
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (post_id, comment_id) = 1)
);

CREATE INDEX idx_person_saved_combined_published ON person_saved_combined (saved DESC, id DESC);

CREATE INDEX idx_person_saved_combined ON person_saved_combined (person_id);

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

