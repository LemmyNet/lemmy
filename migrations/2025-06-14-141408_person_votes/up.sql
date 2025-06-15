ALTER TABLE person_actions
    ADD COLUMN voted_at timestamptz,
    ADD COLUMN upvotes bigint,
    ADD COLUMN downvotes bigint;

ALTER TABLE local_user
    ADD COLUMN show_person_votes boolean NOT NULL DEFAULT FALSE;

-- Adding vote history
-- This union alls the comment and post actions tables,
-- inner joins to local_user for the above to filter out non-locals
-- separates the like_score into upvote and downvote columns,
-- groups and sums the upvotes and downvotes,
-- handles conflicts using the `excluded` magic column.
INSERT INTO person_actions (person_id, target_id, voted_at, upvotes, downvotes)
SELECT
    votes.person_id,
    votes.creator_id,
    now(),
    sum(
        CASE votes.like_score
        WHEN 1 THEN
            1
        ELSE
            0
        END) AS upvotes,
    sum(
        CASE votes.like_score
        WHEN -1 THEN
            1
        ELSE
            0
        END) AS downvotes
FROM (
    SELECT
        pa.person_id,
        p.creator_id,
        like_score
    FROM
        post_actions pa
        INNER JOIN post p ON pa.post_id = p.id
        INNER JOIN local_user lu ON pa.person_id = lu.person_id
UNION ALL
SELECT
    ca.person_id,
    c.creator_id,
    like_score
FROM
    comment_actions ca
    INNER JOIN comment c ON ca.comment_id = c.id
    INNER JOIN local_user lu ON ca.person_id = lu.person_id) AS votes
GROUP BY
    votes.person_id,
    votes.creator_id
ON CONFLICT (person_id,
    target_id)
    DO UPDATE SET
        voted_at = now(),
        upvotes = excluded.upvotes,
        downvotes = excluded.downvotes;

