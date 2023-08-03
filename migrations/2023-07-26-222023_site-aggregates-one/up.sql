CREATE OR REPLACE FUNCTION site_aggregates_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- we only ever want to have a single value in site_aggregate because the site_aggregate triggers update all rows in that table.
    -- a cleaner check would be to insert it for the local_site but that would break assumptions at least in the tests
    IF (TG_OP = 'INSERT') AND NOT EXISTS (
    SELECT
        id
    FROM
        site_aggregates
    LIMIT 1) THEN
        INSERT INTO site_aggregates (site_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM site_aggregates
        WHERE site_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

DELETE FROM site_aggregates a
WHERE NOT EXISTS (
        SELECT
            id
        FROM
            local_site s
        WHERE
            s.site_id = a.site_id);

