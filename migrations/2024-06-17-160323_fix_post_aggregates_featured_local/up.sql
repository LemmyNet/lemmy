-- Fix rows that were not updated because of the old incorrect trigger
UPDATE
    post_aggregates
SET
    featured_local = post.featured_local
FROM
    post
WHERE
    post.id = post_aggregates.post_id
    AND post.featured_local != post_aggregates.featured_local;

