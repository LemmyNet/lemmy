ALTER TABLE site
    ALTER COLUMN require_application SET DEFAULT FALSE;

ALTER TABLE site
    ALTER COLUMN application_question SET DEFAULT NULL;

