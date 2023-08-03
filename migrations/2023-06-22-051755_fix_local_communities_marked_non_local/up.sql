UPDATE
    community c
SET
    local = TRUE
FROM
    local_site ls
    JOIN site s ON ls.site_id = s.id
WHERE
    c.instance_id = s.instance_id
    AND NOT c.local;

