ALTER TABLE multi_community
    ADD COLUMN subscribers int NOT NULL DEFAULT 0,
    ADD COLUMN subscribers_local int NOT NULL DEFAULT 0,
    ADD COLUMN communities int NOT NULL DEFAULT 0;

