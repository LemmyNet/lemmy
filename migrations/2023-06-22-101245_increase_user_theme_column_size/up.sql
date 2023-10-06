ALTER TABLE ONLY local_user
    ALTER COLUMN theme TYPE text;

ALTER TABLE ONLY local_user
    ALTER COLUMN theme SET DEFAULT 'browser'::text;

