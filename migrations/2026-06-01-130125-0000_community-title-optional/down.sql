ALTER TABLE community
    ALTER COLUMN title SET NOT NULL;

ALTER TABLE community_report
    ALTER COLUMN original_community_title SET NOT NULL;

