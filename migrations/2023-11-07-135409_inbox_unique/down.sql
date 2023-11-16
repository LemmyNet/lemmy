ALTER TABLE person
    ADD CONSTRAINT idx_person_inbox_url UNIQUE (inbox_url);

ALTER TABLE community
    ADD CONSTRAINT idx_community_inbox_url UNIQUE (inbox_url);

UPDATE
    site
SET
    inbox_url = inbox_query.inbox
FROM (
    SELECT
        format('https://%s/site_inbox', DOMAIN) AS inbox
    FROM
        instance,
        site,
        local_site
    WHERE
        instance.id = site.instance_id
        AND local_site.id = site.id) AS inbox_query,
    instance,
    local_site
WHERE
    instance.id = site.instance_id
    AND local_site.id = site.id;

