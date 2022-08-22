alter table comment add column language_id integer references language not null default 0;
