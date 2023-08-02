-- Create an instance table
-- Holds any connected or unconnected domain
CREATE TABLE instance (
    id serial PRIMARY KEY,
    domain varchar(255) NOT NULL UNIQUE,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp NULL
);

-- Insert all the domains to the instance table
INSERT INTO instance (DOMAIN)
SELECT DISTINCT
    substring(p.actor_id FROM '(?:.*://)?(?:www\.)?([^/?]*)')
FROM (
    SELECT
        actor_id
    FROM
        site
    UNION
    SELECT
        actor_id
    FROM
        person
    UNION
    SELECT
        actor_id
    FROM
        community) AS p;

-- Alter site, person, and community tables to reference the instance table.
ALTER TABLE site
    ADD COLUMN instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE person
    ADD COLUMN instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE community
    ADD COLUMN instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

-- Add those columns
UPDATE
    site
SET
    instance_id = i.id
FROM
    instance i
WHERE
    substring(actor_id FROM '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

UPDATE
    person
SET
    instance_id = i.id
FROM
    instance i
WHERE
    substring(actor_id FROM '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

UPDATE
    community
SET
    instance_id = i.id
FROM
    instance i
WHERE
    substring(actor_id FROM '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

-- Make those columns unique not null now
ALTER TABLE site
    ALTER COLUMN instance_id SET NOT NULL;

ALTER TABLE site
    ADD CONSTRAINT idx_site_instance_unique UNIQUE (instance_id);

ALTER TABLE person
    ALTER COLUMN instance_id SET NOT NULL;

ALTER TABLE community
    ALTER COLUMN instance_id SET NOT NULL;

-- Create allowlist and blocklist tables
CREATE TABLE federation_allowlist (
    id serial PRIMARY KEY,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL UNIQUE,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp NULL
);

CREATE TABLE federation_blocklist (
    id serial PRIMARY KEY,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL UNIQUE,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp NULL
);

-- Move all the extra site settings-type columns to a local_site table
-- Add a lot of other fields currently in the lemmy.hjson
CREATE TABLE local_site (
    id serial PRIMARY KEY,
    site_id int REFERENCES site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL UNIQUE,
    -- Site table fields
    site_setup boolean DEFAULT FALSE NOT NULL,
    enable_downvotes boolean DEFAULT TRUE NOT NULL,
    open_registration boolean DEFAULT TRUE NOT NULL,
    enable_nsfw boolean DEFAULT TRUE NOT NULL,
    community_creation_admin_only boolean DEFAULT FALSE NOT NULL,
    require_email_verification boolean DEFAULT FALSE NOT NULL,
    require_application boolean DEFAULT TRUE NOT NULL,
    application_question text DEFAULT 'to verify that you are human, please explain why you want to create an account on this site' ::text,
    private_instance boolean DEFAULT FALSE NOT NULL,
    default_theme text DEFAULT 'browser' ::text NOT NULL,
    default_post_listing_type text DEFAULT 'Local' ::text NOT NULL,
    legal_information text,
    hide_modlog_mod_names boolean DEFAULT TRUE NOT NULL,
    application_email_admins boolean DEFAULT FALSE NOT NULL,
    -- Fields from lemmy.hjson
    slur_filter_regex text,
    actor_name_max_length int DEFAULT 20 NOT NULL,
    federation_enabled boolean DEFAULT TRUE NOT NULL,
    federation_debug boolean DEFAULT FALSE NOT NULL,
    federation_strict_allowlist boolean DEFAULT TRUE NOT NULL,
    federation_http_fetch_retry_limit int DEFAULT 25 NOT NULL,
    federation_worker_count int DEFAULT 64 NOT NULL,
    captcha_enabled boolean DEFAULT FALSE NOT NULL,
    captcha_difficulty varchar(255) DEFAULT 'medium' NOT NULL,
    -- Time fields
    published timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

-- local_site_rate_limit is its own table, so as to not go over 32 columns, and force diesel to use the 64-column-tables feature
CREATE TABLE local_site_rate_limit (
    id serial PRIMARY KEY,
    local_site_id int REFERENCES local_site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL UNIQUE,
    message int DEFAULT 180 NOT NULL,
    message_per_second int DEFAULT 60 NOT NULL,
    post int DEFAULT 6 NOT NULL,
    post_per_second int DEFAULT 600 NOT NULL,
    register int DEFAULT 3 NOT NULL,
    register_per_second int DEFAULT 3600 NOT NULL,
    image int DEFAULT 6 NOT NULL,
    image_per_second int DEFAULT 3600 NOT NULL,
    comment int DEFAULT 6 NOT NULL,
    comment_per_second int DEFAULT 600 NOT NULL,
    search int DEFAULT 60 NOT NULL,
    search_per_second int DEFAULT 600 NOT NULL,
    published timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

-- Insert the data into local_site
INSERT INTO local_site (site_id, site_setup, enable_downvotes, open_registration, enable_nsfw, community_creation_admin_only, require_email_verification, require_application, application_question, private_instance, default_theme, default_post_listing_type, legal_information, hide_modlog_mod_names, application_email_admins, published, updated)
SELECT
    id,
    TRUE, -- Assume site if setup if there's already a site row
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
    site
ORDER BY
    id
LIMIT 1;

-- Default here
INSERT INTO local_site_rate_limit (local_site_id)
SELECT
    id
FROM
    local_site
ORDER BY
    id
LIMIT 1;

-- Drop all those columns from site
ALTER TABLE site
    DROP COLUMN enable_downvotes,
    DROP COLUMN open_registration,
    DROP COLUMN enable_nsfw,
    DROP COLUMN community_creation_admin_only,
    DROP COLUMN require_email_verification,
    DROP COLUMN require_application,
    DROP COLUMN application_question,
    DROP COLUMN private_instance,
    DROP COLUMN default_theme,
    DROP COLUMN default_post_listing_type,
    DROP COLUMN legal_information,
    DROP COLUMN hide_modlog_mod_names,
    DROP COLUMN application_email_admins;

