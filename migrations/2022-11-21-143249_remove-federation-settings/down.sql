ALTER TABLE local_site
    ADD COLUMN federation_strict_allowlist bool DEFAULT TRUE NOT NULL;

ALTER TABLE local_site
    ADD COLUMN federation_http_fetch_retry_limit int NOT NULL DEFAULT 25;

