ALTER TABLE modlog
    ADD COLUMN child_count integer NOT NULL DEFAULT 0;

