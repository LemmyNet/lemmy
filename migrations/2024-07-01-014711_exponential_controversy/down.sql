UPDATE
    post_aggregates
SET
    controversy_rank = CASE WHEN downvotes <= 0
        OR upvotes <= 0 THEN
        0
    ELSE
        (upvotes + downvotes) * CASE WHEN upvotes > downvotes THEN
            downvotes::float / upvotes::float
        ELSE
            upvotes::float / downvotes::float
        END
    END
WHERE
    upvotes > 0
    AND downvotes > 0;

