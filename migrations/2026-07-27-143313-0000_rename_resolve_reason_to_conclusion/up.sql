ALTER TABLE post_report RENAME COLUMN resolve_reason TO conclusion;

ALTER TABLE comment_report RENAME COLUMN resolve_reason TO conclusion;

ALTER TABLE community_report RENAME COLUMN resolve_reason TO conclusion;

ALTER TABLE private_message_report RENAME COLUMN resolve_reason TO conclusion;

