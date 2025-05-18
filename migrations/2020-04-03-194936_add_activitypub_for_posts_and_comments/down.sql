ALTER TABLE post
    DROP COLUMN ap_id,
    DROP COLUMN local;

ALTER TABLE comment
    DROP COLUMN ap_id,
    DROP COLUMN local;

