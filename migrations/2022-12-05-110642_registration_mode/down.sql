-- add back old registration columns
ALTER TABLE local_site
    ADD COLUMN open_registration boolean NOT NULL DEFAULT TRUE;

ALTER TABLE local_site
    ADD COLUMN require_application boolean NOT NULL DEFAULT TRUE;

-- regenerate their values
WITH subquery AS (
    SELECT
        registration_mode,
        CASE WHEN registration_mode = 'closed' THEN
            FALSE
        ELSE
            TRUE
        END
    FROM
        local_site)
UPDATE
    local_site
SET
    open_registration = subquery.case
FROM
    subquery;

WITH subquery AS (
    SELECT
        registration_mode,
        CASE WHEN registration_mode = 'open' THEN
            FALSE
        ELSE
            TRUE
        END
    FROM
        local_site)
UPDATE
    local_site
SET
    require_application = subquery.case
FROM
    subquery;

-- drop new column and type
ALTER TABLE local_site
    DROP COLUMN registration_mode;

DROP TYPE registration_mode_enum;

