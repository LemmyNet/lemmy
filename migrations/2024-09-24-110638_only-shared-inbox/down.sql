ALTER TABLE person
    DROP CONSTRAINT person_inbox_id_fkey;

ALTER TABLE person
    DROP COLUMN inbox_url;

DROP TABLE inbox;

ALTER TABLE person add COLUMN inbox_url character varying(255)  not null default generate_unique_changeme();
ALTER TABLE person add COLUMN shared_inbox_url character varying(255)  not null default generate_unique_changeme();

