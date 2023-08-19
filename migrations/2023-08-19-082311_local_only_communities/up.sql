ALTER TABLE community
    ADD COLUMN posting_restricted_to_local boolean DEFAULT FALSE NOT NULL;