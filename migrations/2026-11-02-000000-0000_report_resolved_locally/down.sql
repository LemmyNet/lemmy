DROP INDEX idx_report_combined_published_asc;

CREATE INDEX idx_report_combined_published_asc ON report_combined (resolved, published_at, id);

ALTER TABLE report_combined
    DROP COLUMN resolved_locally;

ALTER TABLE private_message_report
    DROP COLUMN resolved_locally;

ALTER TABLE community_report
    DROP COLUMN resolved_locally;

ALTER TABLE comment_report
    DROP COLUMN resolved_locally;

ALTER TABLE post_report
    DROP COLUMN resolved_locally;

