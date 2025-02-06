ALTER TABLE post_report
    ADD COLUMN to_local_admins bool NOT NULL DEFAULT FALSE;

ALTER TABLE comment_report
    ADD COLUMN to_local_admins bool NOT NULL DEFAULT FALSE;

