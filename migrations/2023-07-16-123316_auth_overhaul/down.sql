-- This file should undo anything in `up.sql`
DROP INDEX idx_auth_api_token_token;
DROP TABLE auth_api_token;
DROP INDEX idx_auth_refresh_token_token;
DROP TABLE auth_refresh_token;
