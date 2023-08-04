ALTER TABLE community
    ADD COLUMN moderators_url varchar(255) UNIQUE;

ALTER TABLE community
    ADD COLUMN featured_url varchar(255) UNIQUE;

