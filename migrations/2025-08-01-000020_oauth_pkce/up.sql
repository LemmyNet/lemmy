ALTER TABLE oauth_provider
    ADD COLUMN use_pkce boolean DEFAULT FALSE NOT NULL;

