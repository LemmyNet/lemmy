alter table password_reset_request add column expires_at timestamp not null;
create index idx_password_reset_request_token_encrypted on password_reset_request using hash (token_encrypted);
