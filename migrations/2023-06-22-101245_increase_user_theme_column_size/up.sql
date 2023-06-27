alter table only local_user alter column theme type text;
alter table only local_user alter column theme set default 'browser'::text;
