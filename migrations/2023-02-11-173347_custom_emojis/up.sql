CREATE TABLE custom_emoji (
    id serial PRIMARY KEY,
    local_site_id int REFERENCES local_site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    shortcode varchar(128) NOT NULL UNIQUE,
    image_url text NOT NULL UNIQUE,
    alt_text text NOT NULL,
    category text NOT NULL,
    published timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

CREATE TABLE custom_emoji_keyword (
    id serial PRIMARY KEY,
    custom_emoji_id int REFERENCES custom_emoji ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    keyword varchar(128) NOT NULL,
    UNIQUE (custom_emoji_id, keyword)
);

CREATE INDEX idx_custom_emoji_category ON custom_emoji (id, category);

