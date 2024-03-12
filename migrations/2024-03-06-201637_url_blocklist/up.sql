CREATE TABLE local_site_url_blocklist (
    id serial NOT NULL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    published timestamp with time zone NOT NULL DEFAULT now(),
    updated timestamp with time zone
);

