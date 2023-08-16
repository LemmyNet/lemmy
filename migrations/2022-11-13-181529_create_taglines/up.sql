CREATE TABLE tagline (
    id serial PRIMARY KEY,
    local_site_id int REFERENCES local_site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    content text NOT NULL,
    published timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

