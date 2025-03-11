UPDATE
    local_site
SET
    users = (
        SELECT
            count(*)
        FROM
            local_user);

