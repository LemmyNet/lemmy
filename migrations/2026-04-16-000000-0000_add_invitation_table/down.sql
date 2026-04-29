ALTER TABLE local_site
    DROP COLUMN max_invites_per_user_allowed;

ALTER TABLE local_user
    DROP COLUMN invited_by_local_user_id;

DROP TABLE local_user_invite;

-- PostgreSQL doesn't support removing enum values directly, so recreate the type.
-- First migrate any RequireInvitation rows to Closed.
UPDATE
    local_site
SET
    registration_mode = 'Closed'
WHERE
    registration_mode = 'RequireInvitation';

CREATE TYPE registration_mode_enum_new AS ENUM (
    'Closed',
    'RequireApplication',
    'Open'
);

ALTER TABLE local_site
    ALTER COLUMN registration_mode DROP DEFAULT;

ALTER TABLE local_site
    ALTER COLUMN registration_mode TYPE registration_mode_enum_new
    USING registration_mode::text::registration_mode_enum_new;

DROP TYPE registration_mode_enum;

ALTER TYPE registration_mode_enum_new RENAME TO registration_mode_enum;

ALTER TABLE local_site
    ALTER COLUMN registration_mode SET DEFAULT 'RequireApplication';

