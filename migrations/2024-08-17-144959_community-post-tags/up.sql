-- a tag for a post, valid in a community. created by mods of a community
CREATE TABLE community_post_tag (
    id serial PRIMARY KEY,
    ap_id text NOT NULL UNIQUE,
    community_id int NOT NULL REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    name text NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz,
    deleted timestamptz
);

-- an association between a post and a community post tag. created/updated by the post author or mods of a community
CREATE TABLE post_community_post_tag (
    post_id int NOT NULL REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    community_post_tag_id int NOT NULL REFERENCES community_post_tag (id) ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (post_id, community_post_tag_id)
);

