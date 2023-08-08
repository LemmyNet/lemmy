ALTER TABLE ONLY local_user
    ALTER COLUMN theme TYPE character varying(20);

ALTER TABLE ONLY local_user
    ALTER COLUMN theme SET DEFAULT 'browser'::character varying;

