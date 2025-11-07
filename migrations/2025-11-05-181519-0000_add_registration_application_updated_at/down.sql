DROP INDEX idx_registration_application_updated;

ALTER TABLE registration_application
    DROP COLUMN updated_at;

