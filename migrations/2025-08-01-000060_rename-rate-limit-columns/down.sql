ALTER TABLE local_site_rate_limit RENAME COLUMN message_max_requests TO message;

ALTER TABLE local_site_rate_limit RENAME COLUMN message_interval_seconds TO message_per_second;

ALTER TABLE local_site_rate_limit RENAME COLUMN post_max_requests TO post;

ALTER TABLE local_site_rate_limit RENAME COLUMN post_interval_seconds TO post_per_second;

ALTER TABLE local_site_rate_limit RENAME COLUMN comment_max_requests TO comment;

ALTER TABLE local_site_rate_limit RENAME COLUMN comment_interval_seconds TO comment_per_second;

ALTER TABLE local_site_rate_limit RENAME COLUMN register_max_requests TO register;

ALTER TABLE local_site_rate_limit RENAME COLUMN register_interval_seconds TO register_per_second;

ALTER TABLE local_site_rate_limit RENAME COLUMN image_max_requests TO image;

ALTER TABLE local_site_rate_limit RENAME COLUMN image_interval_seconds TO image_per_second;

ALTER TABLE local_site_rate_limit RENAME COLUMN search_max_requests TO search;

ALTER TABLE local_site_rate_limit RENAME COLUMN search_interval_seconds TO search_per_second;

ALTER TABLE local_site_rate_limit RENAME COLUMN import_user_settings_max_requests TO import_user_settings;

ALTER TABLE local_site_rate_limit RENAME COLUMN import_user_settings_interval_seconds TO import_user_settings_per_second;

