ALTER TABLE person
    ADD COLUMN shared_inbox_url varchar(255);

ALTER TABLE person RENAME CONSTRAINT person_shared_inbox_url_not_null TO user__inbox_url_not_null;

ALTER TABLE community
    DROP CONSTRAINT community_shared_inbox_url_not_null;

ALTER TABLE community
    ADD COLUMN shared_inbox_url varchar(255),
    ALTER COLUMN inbox_url SET NOT NULL;

