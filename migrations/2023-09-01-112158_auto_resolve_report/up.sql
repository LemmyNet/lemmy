-- Automatically resolve all reports for a given post once it is marked as removed
CREATE OR REPLACE FUNCTION post_removed_resolve_reports ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_report
    SET
        resolved = TRUE,
        resolver_id = NEW.mod_person_id,
        updated = now()
    WHERE
        post_report.post_id = NEW.post_id;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER post_removed_resolve_reports
    AFTER INSERT ON mod_remove_post
    FOR EACH ROW
    WHEN (NEW.removed)
    EXECUTE PROCEDURE post_removed_resolve_reports ();

-- Same when comment is marked as removed
CREATE OR REPLACE FUNCTION comment_removed_resolve_reports ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        comment_report
    SET
        resolved = TRUE,
        resolver_id = NEW.mod_person_id,
        updated = now()
    WHERE
        comment_report.comment_id = NEW.comment_id;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER comment_removed_resolve_reports
    AFTER INSERT ON mod_remove_comment
    FOR EACH ROW
    WHEN (NEW.removed)
    EXECUTE PROCEDURE comment_removed_resolve_reports ();

