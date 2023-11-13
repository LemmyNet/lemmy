-- drop unique constraints for inbox columns
ALTER TABLE person
    DROP CONSTRAINT idx_person_inbox_url;

ALTER TABLE community
    DROP CONSTRAINT idx_community_inbox_url;

-- change site inbox path from /inbox to /site_inbox
-- we dont have any way here to set the correct protocol (http or https) according to tls_enabled, or set
-- the correct port in case of debugging
UPDATE
    site
SET
    inbox_url = inbox_query.inbox
FROM (
    SELECT
        format('https://%s/inbox', DOMAIN) AS inbox
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

