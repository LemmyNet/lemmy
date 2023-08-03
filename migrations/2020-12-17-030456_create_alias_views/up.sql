-- Some view that act as aliases
-- unfortunately necessary, since diesel doesn't have self joins
-- or alias support yet
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

CREATE VIEW comment_alias_1 AS
SELECT
    *
FROM
    comment;

