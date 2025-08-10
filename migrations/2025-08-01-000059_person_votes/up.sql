ALTER TABLE person_actions
    ADD COLUMN voted_at timestamptz,
    ADD COLUMN upvotes int,
    ADD COLUMN downvotes int;

ALTER TABLE local_user
    ADD COLUMN show_person_votes boolean NOT NULL DEFAULT TRUE;

-- Disable the triggers temporarily
ALTER TABLE person_actions DISABLE TRIGGER ALL;

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
    count(*) FILTER (WHERE votes.like_score = 1) AS upvotes,
    count(*) FILTER (WHERE votes.like_score != 1) AS downvotes
FROM (
    SELECT
        pa.person_id,
        p.creator_id,
        like_score
    FROM
        post_actions pa
        INNER JOIN post p ON pa.post_id = p.id
            AND p.local
        UNION ALL
        SELECT
            ca.person_id,
            c.creator_id,
            like_score
        FROM
            comment_actions ca
        INNER JOIN comment c ON ca.comment_id = c.id
            AND c.local) AS votes
GROUP BY
    votes.person_id,
    votes.creator_id
ON CONFLICT (person_id,
    target_id)
    DO UPDATE SET
        voted_at = now(),
        upvotes = excluded.upvotes,
        downvotes = excluded.downvotes;

-- Re-enable the triggers
ALTER TABLE person_actions ENABLE TRIGGER ALL;

