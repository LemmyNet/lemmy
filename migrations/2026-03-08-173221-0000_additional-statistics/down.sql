-- This file should undo anything in `up.sql`
ALTER TABLE local_site
	DROP COLUMN linked_instances;

ALTER TABLE local_site
	DROP COLUMN total_posts;

ALTER TABLE local_site
	DROP COLUMN total_comments;

ALTER TABLE local_site
	DROP COLUMN total_users;

ALTER TABLE local_site
	DROP COLUMN total_communities;

ALTER TABLE local_site
	DROP COLUMN user_retention_percent;

ALTER TABLE local_site
	DROP COLUMN local_post_english_percent;

ALTER TABLE local_site
	DROP COLUMN ban_rate;

ALTER TABLE local_site
	DROP COLUMN accepted_signups_rate;

ALTER TABLE local_site
	DROP COLUMN failed_signups_rate;
