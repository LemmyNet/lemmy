CREATE TABLE user_ (
    id serial PRIMARY KEY,
    name varchar(20) NOT NULL,
    fedi_name varchar(40) NOT NULL,
    preferred_username varchar(20),
    password_encrypted text NOT NULL,
    email text UNIQUE,
    icon bytea,
    admin boolean DEFAULT FALSE NOT NULL,
    banned boolean DEFAULT FALSE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp,
    UNIQUE (name, fedi_name)
);

CREATE TABLE user_ban (
    id serial PRIMARY KEY,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (user_id)
);

INSERT INTO user_ (name, fedi_name, password_encrypted)
    VALUES ('admin', 'TBD', 'TBD');

