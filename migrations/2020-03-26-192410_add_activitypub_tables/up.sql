-- The Activitypub activity table
-- All user actions must create a row here.
CREATE TABLE activity (
    id serial PRIMARY KEY,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- Ensures that the user is set up here.
    data jsonb NOT NULL,
    local boolean NOT NULL DEFAULT TRUE,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

-- Making sure that id is unique
CREATE UNIQUE INDEX idx_activity_unique_apid ON activity ((data ->> 'id'::text));

-- Add federation columns to the two actor tables
ALTER TABLE user_
-- TODO uniqueness constraints should be added on these 3 columns later
    ADD COLUMN actor_id character varying(255) NOT NULL DEFAULT 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
    ADD COLUMN bio text, -- not on community, already has description
    ADD COLUMN local boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN private_key text, -- These need to be generated from code
    ADD COLUMN public_key text,
    ADD COLUMN last_refreshed_at timestamp NOT NULL DEFAULT now() -- Used to re-fetch federated actor periodically
;

-- Community
ALTER TABLE community
    ADD COLUMN actor_id character varying(255) NOT NULL DEFAULT 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
    ADD COLUMN local boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN private_key text, -- These need to be generated from code
    ADD COLUMN public_key text,
    ADD COLUMN last_refreshed_at timestamp NOT NULL DEFAULT now() -- Used to re-fetch federated actor periodically
;

-- Don't worry about rebuilding the views right now.
