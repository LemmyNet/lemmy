ALTER TABLE person
    DROP COLUMN local_banned;

ALTER TABLE person
    DROP COLUMN local_ban_expires;

ALTER TABLE community
    DROP COLUMN local_removed;

ALTER TABLE post
    DROP COLUMN pending;

ALTER TABLE comment
    DROP COLUMN pending;

