ALTER TABLE post_report RENAME COLUMN conclusion TO resolve_reason;

ALTER TABLE comment_report RENAME COLUMN conclusion TO resolve_reason;

ALTER TABLE community_report RENAME COLUMN conclusion TO resolve_reason;

ALTER TABLE private_message_report RENAME COLUMN conclusion TO resolve_reason;

