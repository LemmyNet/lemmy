ALTER TABLE local_site
    ADD COLUMN linked_instances integer NOT NULL DEFAULT 0,
    ADD COLUMN total_posts integer NOT NULL DEFAULT 0,
    ADD COLUMN total_comments integer NOT NULL DEFAULT 0,
    ADD COLUMN total_users integer NOT NULL DEFAULT 0,
    ADD COLUMN total_communities integer NOT NULL DEFAULT 0,
    ADD COLUMN user_retention_month_percent smallint NOT NULL DEFAULT 0,
    ADD COLUMN user_retention_half_year_percent smallint NOT NULL DEFAULT 0,
    ADD COLUMN banned_users_percent smallint NOT NULL DEFAULT 0;

ALTER TABLE local_site RENAME posts TO local_posts;

ALTER TABLE local_site RENAME comments TO local_comments;

ALTER TABLE local_site RENAME users TO local_users;

ALTER TABLE local_site RENAME communities TO local_communities;

-- Percentage of local posts/comments on this instance that use each language
ALTER TABLE LANGUAGE
    ADD COLUMN usage_in_local_posts_percent smallint NOT NULL DEFAULT 0,
    ADD COLUMN usage_in_local_comments_percent smallint NOT NULL DEFAULT 0;

