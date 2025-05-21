CREATE TABLE multi_community (
    id serial PRIMARY KEY,
    creator_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    name varchar(255) NOT NULL,
    title varchar(255),
    description varchar(255),
    deleted bool NOT NULL DEFAULT FALSE,
    ap_id text UNIQUE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz
);

CREATE TABLE multi_community_entry (
    multi_community_id int NOT NULL REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE,
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (multi_community_id, community_id)
);

ALTER TABLE local_site
    ADD COLUMN suggested_communities int REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TYPE listing_type_enum
    ADD VALUE 'Suggested';

