ALTER TABLE local_site_rate_limit RENAME COLUMN message TO message_max_requests;

ALTER TABLE local_site_rate_limit RENAME COLUMN message_per_second TO message_interval_seconds;

ALTER TABLE local_site_rate_limit RENAME COLUMN post TO post_max_requests;

ALTER TABLE local_site_rate_limit RENAME COLUMN post_per_second TO post_interval_seconds;

ALTER TABLE local_site_rate_limit RENAME COLUMN comment TO comment_max_requests;

ALTER TABLE local_site_rate_limit RENAME COLUMN comment_per_second TO comment_interval_seconds;

ALTER TABLE local_site_rate_limit RENAME COLUMN register TO register_max_requests;

ALTER TABLE local_site_rate_limit RENAME COLUMN register_per_second TO register_interval_seconds;

ALTER TABLE local_site_rate_limit RENAME COLUMN image TO image_max_requests;

ALTER TABLE local_site_rate_limit RENAME COLUMN image_per_second TO image_interval_seconds;

ALTER TABLE local_site_rate_limit RENAME COLUMN search TO search_max_requests;

ALTER TABLE local_site_rate_limit RENAME COLUMN search_per_second TO search_interval_seconds;

ALTER TABLE local_site_rate_limit RENAME COLUMN import_user_settings TO import_user_settings_max_requests;

ALTER TABLE local_site_rate_limit RENAME COLUMN import_user_settings_per_second TO import_user_settings_interval_seconds;

