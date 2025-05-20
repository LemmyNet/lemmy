CREATE TABLE post_gallery (
    id serial NOT NULL PRIMARY KEY,
    post_id integer REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    url character varying(2000) NOT NULL,
    page integer NOT NULL DEFAULT 0,
    alt_text text,
    caption character varying(200),
    url_content_type text,
    published timestamp with time zone NOT NULL DEFAULT now()
);

