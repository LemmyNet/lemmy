-- Add the admin_purge tables
CREATE TABLE admin_purge_person (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE admin_purge_community (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE admin_purge_post (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE admin_purge_comment (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

