alter table local_site_rate_limit add column import_user_settings int not null default 1;
alter table local_site_rate_limit add column import_user_settings_per_second int not null default 86400;
