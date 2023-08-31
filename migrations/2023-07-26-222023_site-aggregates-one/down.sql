CREATE OR REPLACE FUNCTION site_aggregates_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO site_aggregates (site_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM site_aggregates
        WHERE site_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

