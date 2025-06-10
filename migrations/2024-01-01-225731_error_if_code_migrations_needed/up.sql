-- https://github.com/LemmyNet/lemmy/pull/5710
-- Uncomment to test:
-- ALTER TABLE site DROP COLUMN instance_id; INSERT INTO site (name, public_key) VALUES ('', '');
DO $$
BEGIN
    IF EXISTS (
        SELECT
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
                    public_key = '')) AS broken_rows) THEN
    RAISE 'Unstable upgrade: Youre on too old a version of lemmy. Upgrade to 0.19.0 first.';
END IF;
END
$$;

