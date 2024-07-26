DROP VIEW user_alias_1, user_alias_2;

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

-- Views are the same as before, except `*` does not reference the dropped columns
CREATE VIEW user_alias_1 AS
SELECT
    *
FROM
    user_;

CREATE VIEW user_alias_2 AS
SELECT
    *
FROM
    user_;

