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
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz
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
    PRIMARY KEY (multi_community_id, person_id)
);

ALTER TABLE local_site
    ADD COLUMN suggested_communities int REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TYPE listing_type_enum
    ADD VALUE 'Suggested';

ALTER TABLE community_actions
    ADD COLUMN is_multi_community_follow bool;

CREATE INDEX idx_multi_community_read_from_name ON multi_community (local, deleted, name);

CREATE INDEX idx_multi_community_ap_id ON multi_community (ap_id);

CREATE INDEX idx_multi_creator_id ON multi_community (creator_id);

CREATE INDEX idx_multi_followed_by ON multi_community_follow (person_id);

