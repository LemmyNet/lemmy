CREATE TABLE multi_community (
    id serial PRIMARY KEY,
    creator_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    instance_id int NOT NULL REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE,
    name varchar(255) NOT NULL,
    title varchar(255),
    description varchar(255),
    local bool NOT NULL DEFAULT TRUE,
    deleted bool NOT NULL DEFAULT FALSE,
    ap_id text UNIQUE NOT NULL DEFAULT generate_unique_changeme (),
    public_key text NOT NULL,
    private_key text,
    inbox_url text NOT NULL DEFAULT generate_unique_changeme (),
    last_refreshed_at timestamptz NOT NULL DEFAULT now(),
    following_url text NOT NULL DEFAULT generate_unique_changeme (),
    published_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

CREATE TABLE multi_community_entry (
    multi_community_id int NOT NULL REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE,
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (multi_community_id, community_id)
);

CREATE TABLE multi_community_follow (
    multi_community_id int NOT NULL REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE,
    person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    follow_state community_follower_state NOT NULL,
    PRIMARY KEY (person_id, multi_community_id)
);

ALTER TABLE local_site
    ADD COLUMN suggested_communities int REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN multi_comm_follower int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- generate new account with randomized name (max 20 chars) and set it
-- as local_site.multi_comm_follower
WITH x AS (
INSERT INTO person (name, public_key, private_key, instance_id, inbox_url, bot_account)
    SELECT
        'multicomm' || substr(gen_random_uuid ()::text, 0, 11),
        public_key,
        private_key,
        instance_id,
        inbox_url,
        TRUE
    FROM
        site,
        local_site
    WHERE
        site.id = local_site.id
    RETURNING
        person.id)
UPDATE
    local_site
SET
    multi_comm_follower = x.id
FROM
    x;

ALTER TABLE local_site
    ALTER COLUMN multi_comm_follower SET NOT NULL;

-- set ap_id for multicomm follower account (should use r.local_url but thats not defined here)
UPDATE
    person
SET
    ap_id = current_setting('lemmy.protocol_and_hostname') || '/u/' || person.name
FROM
    local_site
WHERE
    person.id = local_site.multi_comm_follower;

ALTER TYPE listing_type_enum
    ADD VALUE 'Suggested';

CREATE INDEX idx_multi_community_read_from_name ON multi_community (local)
WHERE
    local AND NOT deleted;

CREATE INDEX idx_multi_community_ap_id ON multi_community (ap_id);

CREATE INDEX idx_multi_creator_id ON multi_community (creator_id);

CREATE INDEX idx_multi_community_follow_multi_id ON multi_community_follow (multi_community_id);

CREATE INDEX idx_multi_community_entry_community_id ON multi_community_entry (community_id);

ALTER TABLE search_combined
    ADD COLUMN multi_community_id int REFERENCES multi_community (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE search_combined
    DROP CONSTRAINT search_combined_check;

ALTER TABLE search_combined
    ADD CONSTRAINT search_combined_check CHECK (num_nonnulls (post_id, comment_id, community_id, person_id, multi_community_id) = 1);

