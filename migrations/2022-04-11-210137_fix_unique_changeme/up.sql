CREATE OR REPLACE FUNCTION generate_unique_changeme ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://changeme.invalid/' || substr(md5(random()::text), 0, 25);
$$;

