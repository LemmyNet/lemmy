drop table language;
drop table local_user_language;

alter table local_user rename column interface_language to lang;
alter table local_user drop column discussion_languages;

alter table post drop column language_id;
