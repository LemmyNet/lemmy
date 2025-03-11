UPDATE
    local_site
SET
    users = users + (
        SELECT
            count(*)
        FROM
            local_user
        WHERE
            NOT accepted_application);

