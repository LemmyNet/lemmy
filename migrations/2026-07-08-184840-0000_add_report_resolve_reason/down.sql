ALTER TABLE private_message_report
    DROP COLUMN resolve_reason;

ALTER TABLE community_report
    DROP COLUMN resolve_reason;

ALTER TABLE comment_report
    DROP COLUMN resolve_reason;

ALTER TABLE post_report
    DROP COLUMN resolve_reason;

