ALTER TABLE local_site_rate_limit
    ADD COLUMN import_user_settings int NOT NULL DEFAULT 1;

ALTER TABLE local_site_rate_limit
    ADD COLUMN import_user_settings_per_second int NOT NULL DEFAULT 86400;

