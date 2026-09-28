ALTER TABLE community
    ALTER COLUMN title DROP NOT NULL;

ALTER TABLE community_report
    ALTER COLUMN original_community_title DROP NOT NULL;

