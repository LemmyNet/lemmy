ALTER TABLE local_site_rate_limit
    ALTER register SET DEFAULT 10;

UPDATE
    local_site_rate_limit
SET
    register = 10
WHERE
    register = 3;

