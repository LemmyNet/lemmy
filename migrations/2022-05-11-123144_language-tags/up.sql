alter table local_user rename column lang to interface_language;
alter table local_user add column discussion_languages varchar(3)[] not null default '{}';

alter table post add column language varchar(3) not null default 'und';
