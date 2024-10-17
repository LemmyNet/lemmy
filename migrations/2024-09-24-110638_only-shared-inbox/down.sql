ALTER TABLE person
    DROP CONSTRAINT person_inbox_id_fkey;

ALTER TABLE person
    DROP COLUMN inbox_url;

DROP TABLE inbox;

ALTER TABLE person
    ADD COLUMN inbox_url character varying(255) NOT NULL DEFAULT generate_unique_changeme ();

ALTER TABLE person
    ADD COLUMN shared_inbox_url character varying(255) NOT NULL DEFAULT generate_unique_changeme ();

