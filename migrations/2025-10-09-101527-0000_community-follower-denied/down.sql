-- revert change to community follow state enum
ALTER TYPE community_follower_state RENAME TO community_follower_state__;

CREATE TYPE community_follower_state AS ENUM (
    'Accepted',
    'Pending',
    'ApprovalRequired'
);

ALTER TABLE community_actions
    ALTER COLUMN follow_state TYPE community_follower_state
    USING follow_state::text::community_follower_state;

ALTER TABLE multi_community_follow
    ALTER COLUMN follow_state TYPE community_follower_state
    USING follow_state::text::community_follower_state;

DROP TYPE community_follower_state__;

