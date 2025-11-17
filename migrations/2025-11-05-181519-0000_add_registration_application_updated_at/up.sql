ALTER TABLE registration_application
    ADD COLUMN updated_at timestamptz;

CREATE INDEX idx_registration_application_updated ON registration_application (updated_at DESC);

