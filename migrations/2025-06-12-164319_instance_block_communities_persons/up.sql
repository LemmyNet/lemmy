-- Currently, the instance.blocked_at columns only blocks communities from the given instance.
--
-- This creates a new block type, to also be able to block persons.
-- Also changes the name of blocked_at to blocked_communities_at
ALTER TABLE instance_actions RENAME COLUMN blocked_at TO blocked_communities_at;

ALTER TABLE instance_actions
    ADD COLUMN blocked_persons_at timestamptz;

