CREATE TABLE local_site_url_blocklist (
       id serial NOT NULL PRIMARY KEY,
       url VARCHAR NOT NULL UNIQUE
);
