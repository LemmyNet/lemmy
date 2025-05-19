CREATE TABLE multi_community (
    id serial PRIMARY KEY,
    owner_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    name text NOT NULL,
    ap_id text UNIQUE NOT NULL
);

CREATE TABLE multi_community_entry (
    multi_community_id int NOT NULL REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE,
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (multi_community_id, community_id)
);

ALTER TABLE local_site
    ADD COLUMN featured_communities int REFERENCES multi_community ON UPDATE CASCADE ON DELETE CASCADE;

