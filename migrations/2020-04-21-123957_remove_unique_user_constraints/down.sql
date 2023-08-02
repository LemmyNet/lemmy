-- The username index
DROP INDEX idx_user_name_lower_actor_id;

CREATE UNIQUE INDEX idx_user_name_lower ON user_ (lower(name));

