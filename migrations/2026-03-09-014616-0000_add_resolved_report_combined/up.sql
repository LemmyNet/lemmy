-- Adds resolved to the report combined table to speed up queries.
ALTER TABLE report_combined
    ADD COLUMN resolved boolean NOT NULL DEFAULT FALSE;

-- post
UPDATE
    report_combined AS rc
SET
    resolved = r.resolved
FROM
    post_report r
WHERE
    rc.post_report_id = r.id;

-- comment
UPDATE
    report_combined AS rc
SET
    resolved = r.resolved
FROM
    comment_report r
WHERE
    rc.comment_report_id = r.id;

-- community
UPDATE
    report_combined AS rc
SET
    resolved = r.resolved
FROM
    community_report r
WHERE
    rc.community_report_id = r.id;

-- private message
UPDATE
    report_combined AS rc
SET
    resolved = r.resolved
FROM
    private_message_report r
WHERE
    rc.private_message_report_id = r.id;

-- For unresolved, its an asc query
DROP INDEX idx_report_combined_published_asc;

CREATE INDEX idx_report_combined_published_asc ON report_combined (resolved, published_at, id);

