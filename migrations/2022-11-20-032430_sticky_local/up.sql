DROP TRIGGER IF EXISTS post_aggregates_stickied ON post;

DROP FUNCTION post_aggregates_stickied;

ALTER TABLE post
    ADD featured_community boolean NOT NULL DEFAULT FALSE;

ALTER TABLE post
    ADD featured_local boolean NOT NULL DEFAULT FALSE;

UPDATE
    post
SET
    featured_community = stickied;

ALTER TABLE post
    DROP COLUMN stickied;

ALTER TABLE post_aggregates
    ADD featured_community boolean NOT NULL DEFAULT FALSE;

ALTER TABLE post_aggregates
    ADD featured_local boolean NOT NULL DEFAULT FALSE;

UPDATE
    post_aggregates
SET
    featured_community = stickied;

ALTER TABLE post_aggregates
    DROP COLUMN stickied;

ALTER TABLE mod_sticky_post RENAME COLUMN stickied TO featured;

ALTER TABLE mod_sticky_post
    ALTER COLUMN featured SET NOT NULL;

ALTER TABLE mod_sticky_post
    ADD is_featured_community boolean NOT NULL DEFAULT TRUE;

ALTER TABLE mod_sticky_post RENAME TO mod_feature_post;

CREATE FUNCTION post_aggregates_featured_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_aggregates pa
    SET
        featured_community = NEW.featured_community
    WHERE
        pa.post_id = NEW.id;
    RETURN NULL;
END
$$;

CREATE FUNCTION post_aggregates_featured_local ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_aggregates pa
    SET
        featured_local = NEW.featured_local
    WHERE
        pa.post_id = NEW.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER post_aggregates_featured_community
    AFTER UPDATE ON public.post
    FOR EACH ROW
    WHEN (old.featured_community IS DISTINCT FROM new.featured_community)
    EXECUTE FUNCTION public.post_aggregates_featured_community ();

CREATE TRIGGER post_aggregates_featured_local
    AFTER UPDATE ON public.post
    FOR EACH ROW
    WHEN (old.featured_local IS DISTINCT FROM new.featured_local)
    EXECUTE FUNCTION public.post_aggregates_featured_local ();

