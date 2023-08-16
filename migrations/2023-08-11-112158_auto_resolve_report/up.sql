-- Automatically resolve all reports for a given post once it is marked as removed
-- TODO: how to set `resolver_id`?
CREATE OR REPLACE FUNCTION post_removed_resolve_reports ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (NEW.removed) THEN
        UPDATE
            post_report
        SET
            resolved = TRUE,
            updated = now()
        WHERE
            post_report.post_id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER post_removed_resolve_reports
    AFTER UPDATE OF removed ON post
    FOR EACH ROW
    WHEN (NEW.removed)
    EXECUTE PROCEDURE post_removed_resolve_reports ();

-- Same when comment is marked as removed
CREATE OR REPLACE FUNCTION comment_removed_resolve_reports ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (NEW.removed) THEN
        UPDATE
            comment_report
        SET
            resolved = TRUE,
            updated = now()
        WHERE
            comment_report.comment_id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER comment_removed_resolve_reports
    AFTER UPDATE OF removed ON comment
    FOR EACH ROW
    WHEN (NEW.removed)
    EXECUTE PROCEDURE comment_removed_resolve_reports ();

