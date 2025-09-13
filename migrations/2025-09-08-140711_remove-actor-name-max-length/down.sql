ALTER TABLE local_site
    ADD COLUMN actor_name_max_length int DEFAULT 20 NOT NULL;

ALTER TABLE person
    ALTER COLUMN display_name TYPE varchar(255);

ALTER TABLE community
    ALTER COLUMN title TYPE varchar(255);

