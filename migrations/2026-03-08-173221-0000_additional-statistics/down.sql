-- This file should undo anything in `up.sql`
ALTER TABLE local_site
    DROP COLUMN linked_instances,
    DROP COLUMN total_posts,
    DROP COLUMN total_comments,
    DROP COLUMN total_users,
    DROP COLUMN total_communities,
    DROP COLUMN user_retention_month_percent,
    DROP COLUMN user_retention_half_year_percent,
    DROP COLUMN ban_rate;

ALTER TABLE local_site RENAME local_posts TO posts;

ALTER TABLE local_site RENAME local_comments TO comments;

ALTER TABLE local_site RENAME local_users TO users;

ALTER TABLE local_site RENAME local_communities TO communities;

ALTER TABLE LANGUAGE
    DROP COLUMN usage_in_local_posts,
    DROP COLUMN usage_in_local_comments;

