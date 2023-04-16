-- Add the post_id column
alter table person_mention add column post_id int references post on update cascade on delete cascade;

-- Make the comment id column nullable
alter table person_mention alter column comment_id drop not null;
