-- https://github.com/LemmyNet/lemmy/pull/5710
SELECT
    id / 0
FROM ((
        SELECT
            id
        FROM
            person
        WHERE
            ap_id LIKE 'http://changeme%'
            OR (local
                AND public_key = ''))
    UNION ALL (
        SELECT
            id
        FROM
            community
        WHERE
            ap_id LIKE 'http://changeme%'
            OR (local
                AND public_key = ''))
    UNION ALL (
        SELECT
            id
        FROM
            post
        WHERE
            thumbnail_url NOT LIKE 'http%'
            OR (local
                AND ap_id LIKE 'http://changeme%'))
    UNION ALL (
        SELECT
            id
        FROM
            comment
        WHERE
            ap_id LIKE 'http://changeme%'
            AND local)
    UNION ALL (
        SELECT
            id
        FROM
            private_message
        WHERE
            ap_id LIKE 'http://changeme%'
            AND local)
    UNION ALL (
        SELECT
            id
        FROM
            site
        WHERE
            public_key = ''));

