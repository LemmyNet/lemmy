-- remove registration mode. removing enum items isnt supported in postgres,
-- so we need to use a workaround
-- https://stackoverflow.com/a/47305844
ALTER TYPE registration_mode_enum RENAME TO registration_mode_enum_old;
CREATE TYPE registration_mode_enum AS ENUM('closed', 'require_application', 'open');
alter table local_site alter column registration_mode drop default;
ALTER TABLE local_site ALTER COLUMN registration_mode TYPE registration_mode_enum USING registration_mode::text::registration_mode_enum;
alter table local_site alter column registration_mode set default 'require_application';
DROP TYPE registration_mode_enum_old;

drop table review_comment;

alter table local_user rename column approved to accepted_application;