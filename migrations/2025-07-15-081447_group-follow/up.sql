CREATE TABLE community_community_follow (
    community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    follower_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (community_id, follower_id)
);

CREATE INDEX idx_community_community_follow_follower ON community_community_follow (follower_id);

