drop table all_languages;

alter table local_user rename column interface_language to lang;
alter table local_user drop column discussion_languages;

alter table post drop column language;
