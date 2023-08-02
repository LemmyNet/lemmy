-- Add back site columns
ALTER TABLE site
    ADD COLUMN enable_downvotes boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN open_registration boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN enable_nsfw boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN community_creation_admin_only boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN require_email_verification boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN require_application boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN application_question text DEFAULT 'to verify that you are human, please explain why you want to create an account on this site'::text,
    ADD COLUMN private_instance boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN default_theme text DEFAULT 'browser'::text NOT NULL,
    ADD COLUMN default_post_listing_type text DEFAULT 'Local'::text NOT NULL,
    ADD COLUMN legal_information text,
    ADD COLUMN hide_modlog_mod_names boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN application_email_admins boolean DEFAULT FALSE NOT NULL;

-- Insert the data back from local_site
UPDATE
    site
SET
    enable_downvotes = ls.enable_downvotes,
    open_registration = ls.open_registration,
    enable_nsfw = ls.enable_nsfw,
    community_creation_admin_only = ls.community_creation_admin_only,
    require_email_verification = ls.require_email_verification,
    require_application = ls.require_application,
    application_question = ls.application_question,
    private_instance = ls.private_instance,
    default_theme = ls.default_theme,
    default_post_listing_type = ls.default_post_listing_type,
    legal_information = ls.legal_information,
    hide_modlog_mod_names = ls.hide_modlog_mod_names,
    application_email_admins = ls.application_email_admins,
    published = ls.published,
    updated = ls.updated
FROM (
    SELECT
        site_id,
        enable_downvotes,
        open_registration,
        enable_nsfw,
        community_creation_admin_only,
        require_email_verification,
        require_application,
        application_question,
        private_instance,
        default_theme,
        default_post_listing_type,
        legal_information,
        hide_modlog_mod_names,
        application_email_admins,
        published,
        updated
    FROM
        local_site) AS ls
WHERE
    site.id = ls.site_id;

-- drop instance columns
ALTER TABLE site
    DROP COLUMN instance_id;

ALTER TABLE person
    DROP COLUMN instance_id;

ALTER TABLE community
    DROP COLUMN instance_id;

DROP TABLE local_site_rate_limit;

DROP TABLE local_site;

DROP TABLE federation_allowlist;

DROP TABLE federation_blocklist;

DROP TABLE instance;

