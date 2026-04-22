ALTER TYPE registration_mode_enum ADD VALUE 'RequireInvitation';

CREATE TYPE local_user_invite_status_enum AS ENUM ('Active', 'Revoked', 'Exhausted', 'Expired');

CREATE TABLE local_user_invite (
  id          serial PRIMARY KEY,
  token text NOT NULL UNIQUE,
  local_user_id  int NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,
  max_uses    int,
  uses_count  int NOT NULL DEFAULT 0,
  expires_at  timestamptz,
  status      local_user_invite_status_enum NOT NULL DEFAULT 'Active',
  published_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE local_user ADD COLUMN invited_by_local_user_id int REFERENCES local_user (id) ON DELETE SET NULL;

