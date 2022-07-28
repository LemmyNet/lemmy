alter table post drop column language_id;
drop table local_user_language;
drop table language;

alter table local_user rename column interface_language to lang;

