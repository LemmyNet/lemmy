CREATE UNIQUE INDEX idx_site_aggregates_1_row_only ON site_aggregates ((TRUE));

-- Drop functions and use `CASCADE` to drop the triggers that use them
DROP FUNCTION comment_aggregates_comment, comment_aggregates_score, community_aggregates_comment_count, community_aggregates_community, community_aggregates_post_count, community_aggregates_post_count_insert, community_aggregates_subscriber_count, delete_follow_before_person, person_aggregates_comment_count, person_aggregates_comment_score, person_aggregates_person, person_aggregates_post_count, person_aggregates_post_insert, person_aggregates_post_score, post_aggregates_comment_count, post_aggregates_featured_community, post_aggregates_featured_local, post_aggregates_post, post_aggregates_score, site_aggregates_comment_delete, site_aggregates_comment_insert, site_aggregates_community_delete, site_aggregates_community_insert, site_aggregates_person_delete, site_aggregates_person_insert, site_aggregates_post_delete, site_aggregates_post_insert, site_aggregates_post_update, site_aggregates_site, was_removed_or_deleted, was_restored_or_created CASCADE;

-- Drop rank functions
DROP FUNCTION controversy_rank, scaled_rank, hot_rank;

-- Defer constraints
ALTER TABLE comment_aggregates
    ALTER CONSTRAINT comment_aggregates_comment_id_fkey INITIALLY DEFERRED;

ALTER TABLE community_aggregates
    ALTER CONSTRAINT community_aggregates_community_id_fkey INITIALLY DEFERRED;

ALTER TABLE person_aggregates
    ALTER CONSTRAINT person_aggregates_person_id_fkey INITIALLY DEFERRED;

ALTER TABLE post_aggregates
    ALTER CONSTRAINT post_aggregates_community_id_fkey INITIALLY DEFERRED,
    ALTER CONSTRAINT post_aggregates_creator_id_fkey INITIALLY DEFERRED,
    ALTER CONSTRAINT post_aggregates_instance_id_fkey INITIALLY DEFERRED,
    ALTER CONSTRAINT post_aggregates_post_id_fkey INITIALLY DEFERRED;

ALTER TABLE site_aggregates
    ALTER CONSTRAINT site_aggregates_site_id_fkey INITIALLY DEFERRED;

-- Fix values that might be incorrect because of the old triggers
UPDATE
    post_aggregates
SET
    featured_local = post.featured_local,
    featured_community = post.featured_community
FROM
    post
WHERE
    post_aggregates.post_id = post.id;

UPDATE
    community_aggregates
SET
    comments = counted.comments
FROM (
    SELECT
        community_id,
        count(*) AS comments
    FROM
        comment,
        LATERAL (
            SELECT
                *
            FROM
                post
            WHERE
                post.id = comment.post_id
            LIMIT 1) AS post
    WHERE
        NOT (comment.deleted
            OR comment.removed
            OR post.deleted
            OR post.removed)
    GROUP BY
        community_id) AS counted
WHERE
    community_aggregates.community_id = counted.community_id;

UPDATE
    site_aggregates
SET
    communities = (
        SELECT
            count(*)
        FROM
            community
        WHERE
            local);

