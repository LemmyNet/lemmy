ALTER TABLE local_site_rate_limit
    ALTER COLUMN import_user_settings_max_requests SET DEFAULT 3;

UPDATE
    local_site_rate_limit
SET
    import_user_settings_max_requests = 3
WHERE
    import_user_settings_max_requests = 1;

