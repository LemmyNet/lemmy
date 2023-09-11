alter table local_user add column totp_2fa_url text;
alter table local_user drop column totp_2fa_enabled;
