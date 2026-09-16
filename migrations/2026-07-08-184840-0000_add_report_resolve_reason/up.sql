ALTER TABLE post_report
    ADD COLUMN resolve_reason text;

ALTER TABLE comment_report
    ADD COLUMN resolve_reason text;

ALTER TABLE community_report
    ADD COLUMN resolve_reason text;

ALTER TABLE private_message_report
    ADD COLUMN resolve_reason text;

