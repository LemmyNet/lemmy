ALTER TABLE instance_actions RENAME COLUMN blocked_communities_at TO blocked_at;

ALTER TABLE instance_actions
    DROP COLUMN blocked_persons_at;

