ALTER TABLE person
    ADD CONSTRAINT idx_person_inbox_url UNIQUE (inbox_url);

ALTER TABLE community
    ADD CONSTRAINT idx_community_inbox_url UNIQUE (inbox_url);

