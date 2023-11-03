-- Change the received_activity table to use less space. Instead of storing the full ap_id url,
-- only store first 32 chars of hash which is enough to catch duplicates. This reduces the size
-- of that column from from ~70 bytes to 33 bytes. Also drop id column which is unnecessary.
-- The `published` time is stored by postgres as unix timestamp, so it only takes 8 bytes already.
ALTER TABLE received_activity
    ADD COLUMN ap_id_hash varchar(32);

UPDATE
    received_activity
SET
    -- cast to string and drop ` \x` from start of hash, limit length
    ap_id_hash = substring(cast(subquery.hash AS text)
        FROM 3 FOR 32)
FROM (
    SELECT
        id,
        digest(a.ap_id, 'sha256') AS hash
    FROM
        received_activity AS a) AS subquery
WHERE
    received_activity.id = subquery.id;

ALTER TABLE received_activity
    DROP COLUMN id;

ALTER TABLE received_activity
    DROP COLUMN ap_id;

ALTER TABLE received_activity
    ADD PRIMARY KEY (ap_id_hash);

