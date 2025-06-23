-- Creates combined tables for
-- person_liked: (comment, post)
--
-- This one is special, because you use the liked date, not the ordinary published
CREATE TABLE person_liked_combined (
    id serial PRIMARY KEY,
    liked timestamptz NOT NULL,
    like_score smallint NOT NULL,
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

CREATE INDEX idx_person_liked_combined_published ON person_liked_combined (liked DESC, id DESC);

CREATE INDEX idx_person_liked_combined ON person_liked_combined (person_id);

-- Updating the history
INSERT INTO person_liked_combined (liked, like_score, person_id, post_id, comment_id)
SELECT
    pa.liked,
    pa.like_score,
    pa.person_id,
    pa.post_id,
    NULL::int
FROM
    post_actions pa
    INNER JOIN person p ON pa.person_id = p.id
WHERE
    pa.liked IS NOT NULL
    AND p.local = TRUE
UNION ALL
SELECT
    ca.liked,
    ca.like_score,
    ca.person_id,
    NULL::int,
    ca.comment_id
FROM
    comment_actions ca
    INNER JOIN person p ON ca.person_id = p.id
WHERE
    liked IS NOT NULL
    AND p.local = TRUE;

-- Make sure only one of the columns is not null
ALTER TABLE person_liked_combined
    ADD CONSTRAINT person_liked_combined_person_id_comment_id_key UNIQUE (person_id, comment_id),
    ADD CONSTRAINT person_liked_combined_person_id_post_id_key UNIQUE (person_id, post_id),
    ADD CONSTRAINT person_liked_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1),
    ALTER CONSTRAINT person_liked_combined_person_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_liked_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_liked_combined_comment_id_fkey NOT DEFERRABLE;

