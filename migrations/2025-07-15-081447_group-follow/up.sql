CREATE TABLE community_community_follow (
    community_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    follower_id int REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (community_id, follower_id)
);

