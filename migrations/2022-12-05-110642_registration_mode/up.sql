-- create enum for registration modes
CREATE TYPE registration_mode_enum AS enum (
    'closed',
    'require_application',
    'open'
);

-- use this enum for registration mode setting
ALTER TABLE local_site
    ADD COLUMN registration_mode registration_mode_enum NOT NULL DEFAULT 'require_application';

-- generate registration mode value from previous settings
WITH subquery AS (
    SELECT
        open_registration,
        require_application,
        CASE WHEN open_registration = FALSE THEN
            'closed'::registration_mode_enum
        WHEN open_registration = TRUE
            AND require_application = TRUE THEN
            'require_application'
        ELSE
            'open'
        END
    FROM
        local_site)
UPDATE
    local_site
SET
    registration_mode = subquery.case
FROM
    subquery;

-- drop old registration settings
ALTER TABLE local_site
    DROP COLUMN open_registration;

ALTER TABLE local_site
    DROP COLUMN require_application;

