alter table person_mention drop column post_id;
alter table person_mention alter column comment_id set not null;
