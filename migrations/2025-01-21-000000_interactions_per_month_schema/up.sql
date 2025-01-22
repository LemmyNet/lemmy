-- First add the interactions_month column
ALTER TABLE community_aggregates
    ADD COLUMN interactions_month bigint NOT NULL DEFAULT 0;

-- Populate initial values for community activity
UPDATE
    community_aggregates ca
SET
    interactions_month = COALESCE((
        SELECT
            sum(comments + upvotes + downvotes)
        FROM post_aggregates pa
        WHERE
            pa.community_id = ca.community_id
            AND pa.published >= date_trunc('month', CURRENT_TIMESTAMP - interval '1 month')), 0);

-- Recompute all scaled rank values using the new function
UPDATE
    post_aggregates pa
SET
    scaled_rank = r.scaled_rank (pa.score, pa.published, (
            SELECT
                interactions_month
            FROM community_aggregates ca
            WHERE
                ca.community_id = pa.community_id));

