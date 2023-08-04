DROP TABLE person_follower;

ALTER TABLE community_follower
    ALTER COLUMN pending DROP NOT NULL;

