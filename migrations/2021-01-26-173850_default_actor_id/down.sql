CREATE OR REPLACE FUNCTION generate_unique_changeme ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'changeme_' || string_agg(substr('abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789', ceil(random() * 62)::integer, 1), '')
    FROM
        generate_series(1, 20)
$$;

