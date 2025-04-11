-- Creates combined tables for
-- person_liked: (comment, post)
--
-- This one is special, because you use the liked date, not the ordinary published
CREATE TABLE person_liked_combined (
    id serial PRIMARY KEY,
    liked timestamptz NOT NULL,
    like_score smallint NOT NULL,
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (post_id, comment_id) = 1)
);

CREATE INDEX idx_person_liked_combined_published ON person_liked_combined (liked DESC, id DESC);

CREATE INDEX idx_person_liked_combined ON person_liked_combined (person_id);

-- Updating the history
INSERT INTO person_liked_combined (liked, like_score, person_id, post_id, comment_id)
SELECT
    liked,
    like_score,
    person_id,
    post_id,
    NULL::int
FROM
    post_actions
WHERE
    liked IS NOT NULL
UNION ALL
SELECT
    liked,
    like_score,
    person_id,
    NULL::int,
    comment_id
FROM
    comment_actions
WHERE
    liked IS NOT NULL;

