ALTER TABLE person_actions
    ADD COLUMN noted_at timestamptz,
    ADD COLUMN note text;

