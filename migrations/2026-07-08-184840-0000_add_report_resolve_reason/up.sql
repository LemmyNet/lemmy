ALTER TABLE post_report
    ADD COLUMN resolve_reason TEXT;

ALTER TABLE comment_report
    ADD COLUMN resolve_reason TEXT;

ALTER TABLE community_report
    ADD COLUMN resolve_reason TEXT;

ALTER TABLE private_message_report
    ADD COLUMN resolve_reason TEXT;

