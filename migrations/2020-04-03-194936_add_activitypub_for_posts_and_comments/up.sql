-- Add federation columns to post, comment
ALTER TABLE post
-- TODO uniqueness constraints should be added on these 3 columns later
    ADD COLUMN ap_id character varying(255) NOT NULL DEFAULT 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
    ADD COLUMN local boolean NOT NULL DEFAULT TRUE;

ALTER TABLE comment
-- TODO uniqueness constraints should be added on these 3 columns later
    ADD COLUMN ap_id character varying(255) NOT NULL DEFAULT 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
    ADD COLUMN local boolean NOT NULL DEFAULT TRUE;

