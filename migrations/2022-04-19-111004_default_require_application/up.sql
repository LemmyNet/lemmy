ALTER TABLE site
    ALTER COLUMN require_application SET DEFAULT TRUE;

ALTER TABLE site
    ALTER COLUMN application_question SET DEFAULT 'To verify that you are human, please explain why you want to create an account on this site';

