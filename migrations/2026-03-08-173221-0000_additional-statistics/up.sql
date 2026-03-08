-- Your SQL goes here
ALTER TABLE local_site
	ADD COLUMN linked_instances integer;

ALTER TABLE local_site
	ADD COLUMN total_posts integer;

ALTER TABLE local_site
	ADD COLUMN total_comments integer; 

ALTER TABLE local_site
	ADD COLUMN total_users integer;

ALTER TABLE local_site
	ADD COLUMN total_communities integer; 

ALTER TABLE local_site
	ADD COLUMN user_retention_percent integer;

ALTER TABLE local_site
	ADD COLUMN local_post_english_percent integer;

ALTER TABLE local_site
	ADD COLUMN ban_rate integer;

ALTER TABLE local_site
	ADD COLUMN accepted_signups_rate integer;

ALTER TABLE local_site
	ADD COLUMN failed_signups_rate integer;
