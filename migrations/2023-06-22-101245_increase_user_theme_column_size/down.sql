alter table only local_user alter column theme TYPE character varying(20);
alter table only local_user alter column theme set default 'browser'::character varying;