ALTER TABLE community
    ALTER COLUMN public_key DROP NOT NULL;

ALTER TABLE person
    ALTER COLUMN public_key DROP NOT NULL;

