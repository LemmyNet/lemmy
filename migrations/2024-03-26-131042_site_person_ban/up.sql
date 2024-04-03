CREATE TABLE site_person_ban
(
    site_id   INTEGER                                NOT NULL
        REFERENCES site
            ON UPDATE CASCADE ON DELETE CASCADE,
    person_id INTEGER                                NOT NULL
        REFERENCES person
            ON UPDATE CASCADE ON DELETE CASCADE,
    published TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    expires   TIMESTAMP WITH TIME ZONE,
    PRIMARY KEY (person_id, site_id)
);

