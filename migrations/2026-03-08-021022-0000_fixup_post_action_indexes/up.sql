-- Remove a pointless hidden index, and add a better one.
DROP INDEX idx_post_actions_hidden_not_null;

CREATE INDEX idx_post_actions_person_hidden ON post_actions (person_id, hidden_at DESC, post_id DESC)
WHERE
    hidden_at IS NOT NULL;

-- Remove 2 pointless read_at indexes, create a better one.
DROP INDEX idx_post_actions_read_not_null, idx_post_actions_on_read_read_not_null;

CREATE INDEX idx_post_actions_person_read ON post_actions (person_id, read_at DESC, post_id DESC)
WHERE
    read_at IS NOT NULL;

