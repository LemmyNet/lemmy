DROP INDEX idx_user_name_lower;

CREATE UNIQUE INDEX idx_user_name_lower_actor_id ON user_ (lower(name), lower(actor_id));

