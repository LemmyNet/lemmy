ALTER TABLE local_site
    ADD COLUMN federation_worker_count int DEFAULT 64 NOT NULL;

