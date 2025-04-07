ALTER TABLE post
    ADD COLUMN disable_reply_notifications bool NOT NULL DEFAULT FALSE;

ALTER TABLE comment
    ADD COLUMN disable_reply_notifications bool NOT NULL DEFAULT FALSE;

