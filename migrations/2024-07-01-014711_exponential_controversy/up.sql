UPDATE
    post_aggregates
SET
    controversy_rank = (upvotes + downvotes) ^ CASE WHEN upvotes > downvotes THEN
        downvotes::float / upvotes::float
    ELSE
        upvotes::float / downvotes::float
    END
WHERE
    upvotes > 0
    AND downvotes > 0
    -- a number divided by itself is 1, and `* 1` does the same thing as `^ 1`
    AND upvotes != downvotes;

