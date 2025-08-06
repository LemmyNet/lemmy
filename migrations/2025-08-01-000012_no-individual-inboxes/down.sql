ALTER TABLE person
    ADD COLUMN shared_inbox_url varchar(255);

ALTER TABLE community
    ADD COLUMN shared_inbox_url varchar(255);

