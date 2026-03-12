-- Creates combined tables for
-- person_content: (comment, post)
-- person_saved: (comment, post)
CREATE TABLE person_content_combined AS
SELECT
    published,
    creator_id,
    id AS post_id,
    NULL::int AS comment_id
FROM
    post
UNION ALL
SELECT
    published,
    creator_id,
    NULL::int,
    id
FROM
    comment;

-- Add the constraints
ALTER TABLE person_content_combined
    ADD COLUMN id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN creator_id SET NOT NULL,
    ADD CONSTRAINT person_content_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_content_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_content_combined_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD UNIQUE (post_id),
    ADD UNIQUE (comment_id),
    ADD CONSTRAINT person_content_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

CREATE INDEX idx_person_content_combined_creator_published ON person_content_combined (creator_id, published DESC, id DESC);

-- This is for local_users only
CREATE TABLE person_saved_combined AS
SELECT
    pa.saved AS saved,
    pa.person_id AS person_id,
    p.creator_id AS creator_id,
    pa.post_id AS post_id,
    NULL::int AS comment_id
FROM
    post_actions pa,
    local_user lu,
    post p
WHERE
    pa.person_id = lu.person_id
    AND pa.saved IS NOT NULL
    AND pa.post_id = p.id
UNION ALL
SELECT
    ca.saved,
    ca.person_id,
    c.creator_id AS creator_id,
    NULL::int,
    ca.comment_id
FROM
    comment_actions ca,
    local_user lu,
    comment c
WHERE
    ca.person_id = lu.person_id
    AND ca.saved IS NOT NULL
    AND ca.comment_id = c.id;

-- Add the constraints
ALTER TABLE person_saved_combined
    ADD COLUMN id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    ALTER COLUMN saved SET NOT NULL,
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN creator_id SET NOT NULL,
    ADD CONSTRAINT person_saved_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_saved_combined_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_saved_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_saved_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_saved_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1),
    ADD UNIQUE (person_id, post_id),
    ADD UNIQUE (person_id, comment_id);

CREATE INDEX idx_person_saved_combined_person_saved ON person_saved_combined (person_id, saved DESC, id DESC);

CREATE INDEX idx_person_saved_combined_person ON person_saved_combined (person_id);

CREATE INDEX idx_person_saved_combined_creator ON person_saved_combined (creator_id);

