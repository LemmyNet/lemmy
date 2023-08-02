-- Add case insensitive username and email uniqueness
-- An example of showing the dupes:
-- select
--   max(id) as id,
--   lower(name) as lname,
--   count(*)
-- from user_
-- group by lower(name)
-- having count(*) > 1;
-- Delete username dupes, keeping the first one
DELETE FROM user_
WHERE id NOT IN (
        SELECT
            min(id)
        FROM
            user_
        GROUP BY
            lower(name),
            lower(fedi_name));

-- The user index
CREATE UNIQUE INDEX idx_user_name_lower ON user_ (lower(name));

-- Email lower
CREATE UNIQUE INDEX idx_user_email_lower ON user_ (lower(email));

-- Set empty emails properly to null
UPDATE
    user_
SET
    email = NULL
WHERE
    email = '';

