alter table local_user drop column totp_2fa_url;
alter table local_user add column totp_2fa_enabled boolean not null default false;
