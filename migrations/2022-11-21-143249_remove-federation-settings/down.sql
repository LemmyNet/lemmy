alter table local_site add column federation_strict_allowlist bool default true not null;
alter table local_site add column federation_http_fetch_retry_limit int not null default 25;
