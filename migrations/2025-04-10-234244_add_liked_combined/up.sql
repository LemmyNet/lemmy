-- Creates combined tables for
-- person_liked: (comment, post)
SET session_replication_role = REPLICA;

-- This one is special, because you use the liked date, not the ordinary published
CREATE TABLE person_liked_combined (
    id serial PRIMARY KEY,
    liked timestamptz NOT NULL,
    like_score smallint NOT NULL,
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    UNIQUE (person_id, comment_id),
    UNIQUE (person_id, post_id)
);

CREATE INDEX idx_person_liked_combined_published ON person_liked_combined (liked DESC, id DESC);

CREATE INDEX idx_person_liked_combined ON person_liked_combined (person_id);

-- Updating the history
INSERT INTO person_liked_combined (liked, like_score, person_id, post_id)
SELECT
    pa.liked,
    pa.like_score,
    pa.person_id,
    pa.post_id
FROM
    post_actions pa
    INNER JOIN person p ON pa.person_id = p.id
WHERE
    pa.liked IS NOT NULL
    AND p.local = TRUE
    AND pa.liked > CURRENT_DATE - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        liked = excluded.liked,
        like_score = excluded.like_score;

-- Leave the last_scanned_ids blank, since post_actions might not be filled yet.
-- You need the post_actions table to be filled entirely before filling this history.
INSERT INTO history_status (source, dest)
    VALUES ('post_actions', 'person_liked_combined');

INSERT INTO person_liked_combined (liked, like_score, person_id, comment_id)
SELECT
    ca.liked,
    ca.like_score,
    ca.person_id,
    ca.comment_id
FROM
    comment_actions ca
    INNER JOIN person p ON ca.person_id = p.id
WHERE
    liked IS NOT NULL
    AND p.local = TRUE
    AND ca.liked > CURRENT_DATE - interval '1 month'
ON CONFLICT (person_id,
    comment_id)
    DO UPDATE SET
        liked = excluded.liked,
        like_score = excluded.like_score;

INSERT INTO history_status (source, dest)
    VALUES ('comment_actions', 'person_liked_combined');

-- Make sure only one of the columns is not null
ALTER TABLE person_liked_combined
    ADD CONSTRAINT person_liked_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1),
    ALTER CONSTRAINT person_liked_combined_person_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_liked_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_liked_combined_comment_id_fkey NOT DEFERRABLE;

