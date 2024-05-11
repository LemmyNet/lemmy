drop schema if exists r cascade;
-- `content` can't be used as primary key because of size limit
create table previously_run_sql (id boolean primary key, content text);
insert into previously_run_sql (id, content) values (true, '');

