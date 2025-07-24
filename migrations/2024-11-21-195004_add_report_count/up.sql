-- Adding report_count and unresolved_report_count
-- to the post and comment aggregate tables
ALTER TABLE post_aggregates
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0;

-- Disable the triggers temporarily
ALTER TABLE post_aggregates DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'post_aggregates');

-- Update the historical counts
-- Posts
UPDATE
    post_aggregates AS a
SET
    report_count = cnt.count
FROM (
    SELECT
        post_id,
        count(*) AS count
    FROM
        post_report
    GROUP BY
        post_id) cnt
WHERE
    a.post_id = cnt.post_id;

-- The unresolved
UPDATE
    post_aggregates AS a
SET
    unresolved_report_count = cnt.count
FROM (
    SELECT
        post_id,
        count(*) AS count
    FROM
        post_report
    WHERE
        resolved = 'f'
    GROUP BY
        post_id) cnt
WHERE
    a.post_id = cnt.post_id;

-- Re-enable triggers after upserts
ALTER TABLE post_aggregates ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'post_aggregates');

-- reindex
REINDEX TABLE post_aggregates;

ALTER TABLE comment_aggregates
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0;

-- Disable the triggers temporarily
ALTER TABLE comment_aggregates DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'comment_aggregates');

-- Comments
UPDATE
    comment_aggregates AS a
SET
    report_count = cnt.count
FROM (
    SELECT
        comment_id,
        count(*) AS count
    FROM
        comment_report
    GROUP BY
        comment_id) cnt
WHERE
    a.comment_id = cnt.comment_id;

-- The unresolved
UPDATE
    comment_aggregates AS a
SET
    unresolved_report_count = cnt.count
FROM (
    SELECT
        comment_id,
        count(*) AS count
    FROM
        comment_report
    WHERE
        resolved = 'f'
    GROUP BY
        comment_id) cnt
WHERE
    a.comment_id = cnt.comment_id;

-- Re-enable triggers after upserts
ALTER TABLE comment_aggregates ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'comment_aggregates');

-- reindex
REINDEX TABLE comment_aggregates;

