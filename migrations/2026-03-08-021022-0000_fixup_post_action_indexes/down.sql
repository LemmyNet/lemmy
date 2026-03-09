DROP INDEX idx_post_actions_person_hidden;

CREATE INDEX idx_post_actions_hidden_not_null ON post_actions (person_id, post_id)
WHERE
    hidden_at IS NOT NULL;

DROP INDEX idx_post_actions_person_read;

CREATE INDEX idx_post_actions_read_not_null ON post_actions (person_id, post_id)
WHERE
    read_at IS NOT NULL;

CREATE INDEX idx_post_actions_on_read_read_not_null ON post_actions (person_id, read_at, post_id)
WHERE
    read_at IS NOT NULL;

