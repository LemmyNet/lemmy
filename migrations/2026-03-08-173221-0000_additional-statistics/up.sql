-- Your SQL goes here
ALTER TABLE local_site
    ADD COLUMN linked_instances integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN total_posts integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN total_comments integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN total_users integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN total_communities integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN user_retention_percent integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN ban_rate integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN accepted_signups_rate integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN failed_signups_rate integer NOT NULL DEFAULT 0;

ALTER TABLE local_site
    ADD COLUMN language_usage_percent jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE local_site RENAME posts TO local_posts;

ALTER TABLE local_site RENAME comments TO local_comments;

ALTER TABLE local_site RENAME users TO local_users;

ALTER TABLE local_site RENAME communities TO local_communities;

