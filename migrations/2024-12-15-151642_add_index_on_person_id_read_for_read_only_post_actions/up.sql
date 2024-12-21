CREATE INDEX idx_post_actions_on_read_read_not_null ON post_actions (person_id, read, post_id)
WHERE
    read IS NOT NULL;

