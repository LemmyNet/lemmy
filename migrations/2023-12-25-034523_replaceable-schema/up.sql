-- Drop functions and use `CASCADE` to drop the triggers that use them
DROP FUNCTION comment_aggregates_comment, comment_aggregates_score, comment_removed_resolve_reports, community_aggregates_comment_count, community_aggregates_community, community_aggregates_post_count, community_aggregates_post_count_insert, community_aggregates_subscriber_count, person_aggregates_comment_count, person_aggregates_comment_score, person_aggregates_person, person_aggregates_post_count, person_aggregates_post_insert, person_aggregates_post_score, post_aggregates_comment_count, post_aggregates_featured_community, post_aggregates_featured_local, post_aggregates_post, post_aggregates_score, post_removed_resolve_reports, site_aggregates_comment_delete, site_aggregates_comment_insert, site_aggregates_community_insert, site_aggregates_person_delete, site_aggregates_person_insert, site_aggregates_post_delete, site_aggregates_post_insert, site_aggregates_post_update, site_aggregates_site, was_removed_or_deleted, was_restored_or_created CASCADE;

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
        community.id AS community_id,
        count(*) AS comments
    FROM
        comment,
    WHERE
        NOT (comment.deleted
            OR comment.removed
            OR EXISTS (
                SELECT
                    1
                FROM
                    post
                WHERE
                    post.id = comment.post_id
                    AND (post.deleted
                        OR post.removed)))
        GROUP BY
            community.id) AS counted
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

