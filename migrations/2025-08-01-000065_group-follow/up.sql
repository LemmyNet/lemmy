CREATE TABLE community_community_follow (
    target_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (community_id, target_id)
);

CREATE INDEX idx_community_community_follow_target ON community_community_follow (target_id);

