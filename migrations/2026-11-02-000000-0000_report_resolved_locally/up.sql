-- `resolved` is shared with incoming ActivityPub resolves. `resolved_locally` is only set by a
-- local admin/mod action. Existing rows are copied from `resolved` so already-closed reports do
-- not reappear in the local queue; we cannot tell which of those were closed remotely.
ALTER TABLE post_report
    ADD COLUMN resolved_locally bool NOT NULL DEFAULT FALSE;

UPDATE
    post_report
SET
    resolved_locally = resolved;

ALTER TABLE comment_report
    ADD COLUMN resolved_locally bool NOT NULL DEFAULT FALSE;

UPDATE
    comment_report
SET
    resolved_locally = resolved;

ALTER TABLE community_report
    ADD COLUMN resolved_locally bool NOT NULL DEFAULT FALSE;

UPDATE
    community_report
SET
    resolved_locally = resolved;

ALTER TABLE private_message_report
    ADD COLUMN resolved_locally bool NOT NULL DEFAULT FALSE;

UPDATE
    private_message_report
SET
    resolved_locally = resolved;

ALTER TABLE report_combined
    ADD COLUMN resolved_locally bool NOT NULL DEFAULT FALSE;

UPDATE
    report_combined
SET
    resolved_locally = resolved;

-- The unresolved queue now filters on `resolved_locally`, oldest first.
DROP INDEX idx_report_combined_published_asc;

CREATE INDEX idx_report_combined_published_asc ON report_combined (resolved_locally, published_at, id);

