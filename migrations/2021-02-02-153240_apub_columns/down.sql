ALTER TABLE community
    DROP COLUMN followers_url;

ALTER TABLE community
    DROP COLUMN inbox_url;

ALTER TABLE community
    DROP COLUMN shared_inbox_url;

ALTER TABLE user_
    DROP COLUMN inbox_url;

ALTER TABLE user_
    DROP COLUMN shared_inbox_url;

