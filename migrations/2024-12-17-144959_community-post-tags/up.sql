-- a tag for a post, valid in a community. created by mods of a community
CREATE TABLE tag (
    id serial PRIMARY KEY,
    ap_id text NOT NULL UNIQUE,
    name text NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz,
    deleted timestamptz
);

-- indicates this tag was created by the mod of a community and can be applied to posts in this community
CREATE TABLE community_post_tag (
    community_id int NOT NULL REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    tag_id int NOT NULL REFERENCES tag (id) ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (community_id, tag_id)
);

-- an association between a post and a tag. created/updated by the post author or mods of a community
CREATE TABLE post_tag (
    post_id int NOT NULL REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    tag_id int NOT NULL REFERENCES tag (id) ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (post_id, tag_id)
);

