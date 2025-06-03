UPDATE
    site_aggregates
SET
    users = (
        SELECT
            count(*)
        FROM
            local_user);

