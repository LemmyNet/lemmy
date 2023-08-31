-- Delete the empty ap_ids
DELETE FROM activity
WHERE ap_id IS NULL;

-- Make it required
ALTER TABLE activity
    ALTER COLUMN ap_id SET NOT NULL;

-- Delete dupes, keeping the first one
DELETE FROM activity a USING (
    SELECT
        min(id) AS id,
        ap_id
    FROM
        activity
    GROUP BY
        ap_id
    HAVING
        count(*) > 1) b
WHERE
    a.ap_id = b.ap_id
    AND a.id <> b.id;

-- The index
CREATE UNIQUE INDEX idx_activity_ap_id ON activity (ap_id);

-- Drop the old index
DROP INDEX idx_activity_unique_apid;

