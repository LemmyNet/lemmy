CREATE TABLE community_community_follow (
    target_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (target_id, community_id)
);

CREATE INDEX idx_community_community_follow_follower ON community_community_follow (community_id);

