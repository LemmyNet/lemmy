ALTER TABLE local_site
    add column enable_nsfw boolean not null default false;
    
UPDATE
    local_site
SET
enable_nsfw = CASE WHEN site.content_warning is null THEN
        false
    ELSE
        true
    END
FROM
    site
WHERE
    -- only local site has private key
    site.private_key IS NOT NULL;