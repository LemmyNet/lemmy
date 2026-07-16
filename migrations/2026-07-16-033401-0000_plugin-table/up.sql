CREATE TABLE plugin (
    id serial PRIMARY KEY,
    file text NOT NULL UNIQUE,
    hash text NOT NULL,
    allowed_hosts text
);

CREATE TABLE plugin_config (
    id serial PRIMARY KEY,
    plugin_id int REFERENCES plugin ON DELETE CASCADE ON UPDATE CASCADE NOT NULL,
    key text NOT NULL,
    value text NOT NULL
);

ALTER TABLE plugin_config
    ADD CONSTRAINT idx_plugin_config_key_unique UNIQUE (plugin_id, key);

