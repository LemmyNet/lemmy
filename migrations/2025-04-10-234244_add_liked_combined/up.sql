-- Creates combined tables for
-- person_liked: (comment, post)
-- This one is special, because you use the liked date, not the ordinary published
-- Updating the history
CREATE TABLE person_liked_combined AS
SELECT
    pa.liked,
    NULL::int AS id,
    pa.person_id,
    pa.post_id,
    NULL::int AS comment_id,
    pa.like_score_is_positive
FROM
    post_actions pa
    INNER JOIN person p ON pa.person_id = p.id
WHERE
    pa.liked IS NOT NULL
    AND p.local = TRUE
UNION ALL
SELECT
    ca.liked,
    NULL::int,
    ca.person_id,
    NULL::int,
    ca.comment_id,
    ca.like_score_is_positive
FROM
    comment_actions ca
    INNER JOIN person p ON ca.person_id = p.id
WHERE
    liked IS NOT NULL
    AND p.local = TRUE;

ALTER TABLE person_liked_combined
    ALTER COLUMN id
    ADD GENERATED ALWAYS AS IDENTITY,
    ALTER COLUMN liked SET NOT NULL,
    ALTER COLUMN like_score_is_positive SET NOT NULL,
    ALTER COLUMN person_id SET NOT NULL,
    ADD CONSTRAINT person_liked_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_liked_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_liked_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    ADD UNIQUE (person_id, post_id),
    ADD UNIQUE (person_id, comment_id),
    ADD PRIMARY KEY (id),
    ADD CONSTRAINT person_liked_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

CREATE INDEX idx_person_liked_combined ON person_liked_combined (person_id);

