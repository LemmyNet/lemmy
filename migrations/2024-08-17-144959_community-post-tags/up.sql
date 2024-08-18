-- a tag for a post, valid in a community. created by mods of a community
CREATE TABLE community_post_tag (
  id SERIAL PRIMARY KEY,
  ap_id TEXT NOT NULL UNIQUE,
  community_id INT NOT NULL REFERENCES community(id),
  name TEXT NOT NULL,
  published TIMESTAMPTZ NOT NULL,
  updated TIMESTAMPTZ,
  deleted TIMESTAMPTZ
);

-- an association between a post and a community post tag. created/updated by the post author or mods of a community
CREATE TABLE post_community_post_tag (
  post_id INT NOT NULL REFERENCES post(id),
  community_post_tag_id INT NOT NULL REFERENCES community_post_tag(id),
  PRIMARY KEY (post_id, community_post_tag_id)
);
