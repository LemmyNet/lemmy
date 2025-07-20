-- Creates combined tables for
-- person_content: (comment, post)
-- person_saved: (comment, post)
CREATE TABLE person_content_combined (
    id int GENERATED ALWAYS AS IDENTITY,
    published timestamptz NOT NULL,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

-- Disable the triggers temporarily
ALTER TABLE person_content_combined DISABLE TRIGGER ALL;

INSERT INTO person_content_combined (published, post_id, comment_id)
SELECT
    published,
    id AS post_id,
    NULL::int AS comment_id
FROM
    post
UNION ALL
SELECT
    published,
    NULL::int,
    id
FROM
    comment;

-- Re-enable triggers after upserts
ALTER TABLE person_content_combined ENABLE TRIGGER ALL;

-- add the primary key
ALTER TABLE person_content_combined
    ADD PRIMARY KEY (id);

-- This one is special, because you use the saved date, not the ordinary published
CREATE TABLE person_saved_combined (
    id int GENERATED ALWAYS AS IDENTITY,
    saved timestamptz NOT NULL,
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

-- Disable the triggers temporarily
ALTER TABLE person_saved_combined DISABLE TRIGGER ALL;

-- This is for local_users only
INSERT INTO person_saved_combined (saved, person_id, post_id, comment_id)
SELECT
    pa.saved,
    pa.person_id,
    pa.post_id,
    NULL::int AS comment_id
FROM
    post_actions pa,
    local_user lu
WHERE
    pa.person_id = lu.person_id
    AND pa.saved IS NOT NULL
UNION ALL
SELECT
    ca.saved,
    ca.person_id,
    NULL::int,
    ca.comment_id
FROM
    comment_actions ca,
    local_user lu
WHERE
    ca.person_id = lu.person_id
    AND ca.saved IS NOT NULL;

-- add the primary key
ALTER TABLE person_saved_combined
    ADD PRIMARY KEY (id);

CREATE INDEX idx_person_saved_combined_published ON person_saved_combined (saved DESC, id DESC);

CREATE INDEX idx_person_saved_combined ON person_saved_combined (person_id);

ALTER TABLE person_saved_combined
    ALTER CONSTRAINT person_saved_combined_person_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_saved_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_saved_combined_comment_id_fkey NOT DEFERRABLE,
    ADD CONSTRAINT person_saved_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1),
    ADD CONSTRAINT person_saved_combined_person_post_uniq UNIQUE (person_id, post_id),
    ADD CONSTRAINT person_saved_combined_person_comment_uniq UNIQUE (person_id, comment_id);

ALTER TABLE person_content_combined
    ALTER CONSTRAINT person_content_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_content_combined_comment_id_fkey NOT DEFERRABLE,
    ADD CONSTRAINT person_content_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

