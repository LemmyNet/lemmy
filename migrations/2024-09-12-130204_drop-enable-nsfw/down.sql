ALTER TABLE local_site
    ADD COLUMN enable_nsfw boolean NOT NULL DEFAULT FALSE;

UPDATE
    local_site
SET
    enable_nsfw = CASE WHEN site.content_warning IS NULL THEN
        FALSE
    ELSE
        TRUE
    END
FROM
    site
WHERE
    -- only local site has private key
    site.private_key IS NOT NULL;

