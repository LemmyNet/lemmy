ALTER TABLE local_site_rate_limit
    ALTER register SET DEFAULT 3;

UPDATE
    local_site_rate_limit
SET
    register = 3
WHERE
    register = 10;

