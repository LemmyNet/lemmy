-- if site has enable_nsfw, set a default content warning
UPDATE
    site
SET
    content_warning = CASE WHEN local_site.enable_nsfw THEN
        'nsfw'
    ELSE
        NULL
    END
FROM
    local_site
    -- only local site has private key
WHERE
    private_key IS NOT NULL
    -- dont overwrite existing content warning
    AND content_warning IS NOT NULL;

ALTER TABLE local_site
    DROP enable_nsfw;
