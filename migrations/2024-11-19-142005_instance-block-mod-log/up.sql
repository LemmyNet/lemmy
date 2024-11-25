ALTER TABLE federation_blocklist
    ADD COLUMN expires timestamptz;

CREATE TABLE admin_block_instance (
    id serial PRIMARY KEY,
    instance_id int NOT NULL REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE,
    admin_person_id int NOT NULL REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    blocked bool NOT NULL,
    reason text,
    expires timestamptz,
    published timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE admin_allow_instance (
    id serial PRIMARY KEY,
    instance_id int NOT NULL REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE,
    admin_person_id int NOT NULL REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    allowed bool NOT NULL,
    reason text,
    published timestamptz NOT NULL DEFAULT now()
);

