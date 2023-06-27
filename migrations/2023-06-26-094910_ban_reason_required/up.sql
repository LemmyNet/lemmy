-- Make ban reason required in existing table
alter table mod_ban alter column reason set not null;
alter table mod_ban_from_community alter column reason set not null;