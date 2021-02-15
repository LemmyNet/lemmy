-- Person
-- Drop the 2 views user_alias_1, user_alias_2
drop view user_alias_1, user_alias_2;

-- rename the user_ table to person
alter table user_ rename to person;
alter sequence user__id_seq rename to person_id_seq;

-- create a new table local_user
create table local_user (
  id serial primary key,
  user_id int references person on update cascade on delete cascade not null,
  password_encrypted text not null,
  email text,
  admin boolean default false not null,
  show_nsfw boolean default false not null,
  theme character varying(20) default 'darkly'::character varying not null,
  default_sort_type smallint default 0 not null,
  default_listing_type smallint default 1 not null,
  lang character varying(20) default 'browser'::character varying not null,
  show_avatars boolean default true not null,
  send_notifications_to_email boolean default false not null,
  matrix_user_id text,
  unique (user_id)
);

-- Copy the local users over to the new table
insert into local_user 
(
  user_id,
  password_encrypted,
  email,
  admin,
  show_nsfw,
  theme,
  default_sort_type,
  default_listing_type,
  lang,
  show_avatars,
  send_notifications_to_email,
  matrix_user_id
)
select
  id,
  password_encrypted,
  email,
  admin,
  show_nsfw,
  theme,
  default_sort_type,
  default_listing_type,
  lang,
  show_avatars,
  send_notifications_to_email,
  matrix_user_id
from person
where local = true;

-- Drop those columns from person
alter table person 
  drop column password_encrypted,
  drop column email,
  drop column admin,
  drop column show_nsfw,
  drop column theme,
  drop column default_sort_type,
  drop column default_listing_type,
  drop column lang,
  drop column show_avatars,
  drop column send_notifications_to_email,
  drop column matrix_user_id;

-- Rename indexes
alter index user__pkey rename to person__pkey;
alter index idx_user_actor_id rename to idx_person_actor_id;
alter index idx_user_inbox_url rename to idx_person_inbox_url;
alter index idx_user_lower_actor_id rename to idx_person_lower_actor_id;
alter index idx_user_published rename to idx_person_published;

-- Rename triggers
alter trigger site_aggregates_user_delete on person rename to site_aggregates_person_delete;
alter trigger site_aggregates_user_insert on person rename to site_aggregates_person_insert;

-- Rename the trigger functions
alter function site_aggregates_user_delete() rename to site_aggregates_person_delete;
alter function site_aggregates_user_insert() rename to site_aggregates_person_insert;

-- Create views
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;

-- Redo user aggregates into person_aggregates
alter table user_aggregates rename to person_aggregates;
alter sequence user_aggregates_id_seq rename to person_aggregates_id_seq;
alter table person_aggregates rename column user_id to person_id;

-- index
alter index user_aggregates_pkey rename to person_aggregates_pkey;
alter index idx_user_aggregates_comment_score rename to idx_person_aggregates_comment_score;
alter index user_aggregates_user_id_key rename to person_aggregates_person_id_key;
alter table person_aggregates rename constraint user_aggregates_user_id_fkey to person_aggregates_person_id_fkey;


-- Drop all the old triggers and functions
drop trigger user_aggregates_user on person;
drop trigger user_aggregates_post_count on post;
drop trigger user_aggregates_post_score on post_like;
drop trigger user_aggregates_comment_count on comment;
drop trigger user_aggregates_comment_score on comment_like;
drop function 
  user_aggregates_user, 
  user_aggregates_post_count,
  user_aggregates_post_score,
  user_aggregates_comment_count,
  user_aggregates_comment_score;

-- initial user add
create function person_aggregates_person()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into person_aggregates (person_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from person_aggregates where person_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_person
after insert or delete on person
for each row
execute procedure person_aggregates_person();

-- post count
create function person_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates 
    set post_count = post_count + 1 where person_id = NEW.creator_id;

  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates 
    set post_count = post_count - 1 where person_id = OLD.creator_id;

    -- If the post gets deleted, the score calculation trigger won't fire, 
    -- so you need to re-calculate
    update person_aggregates ua
    set post_score = pd.score
    from (
      select u.id,
      coalesce(0, sum(pl.score)) as score
      -- User join because posts could be empty
      from person u 
      left join post p on u.id = p.creator_id
      left join post_like pl on p.id = pl.post_id
      group by u.id
    ) pd 
    where ua.person_id = OLD.creator_id;

  END IF;
  return null;
end $$;

create trigger person_aggregates_post_count
after insert or delete on post
for each row
execute procedure person_aggregates_post_count();

-- post score
create function person_aggregates_post_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update person_aggregates ua
    set post_score = post_score + NEW.score
    from post p
    where ua.person_id = p.creator_id and p.id = NEW.post_id;
    
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates ua
    set post_score = post_score - OLD.score
    from post p
    where ua.person_id = p.creator_id and p.id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_post_score
after insert or delete on post_like
for each row
execute procedure person_aggregates_post_score();

-- comment count
create function person_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates 
    set comment_count = comment_count + 1 where person_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates 
    set comment_count = comment_count - 1 where person_id = OLD.creator_id;

    -- If the comment gets deleted, the score calculation trigger won't fire, 
    -- so you need to re-calculate
    update person_aggregates ua
    set comment_score = cd.score
    from (
      select u.id,
      coalesce(0, sum(cl.score)) as score
      -- User join because comments could be empty
      from person u 
      left join comment c on u.id = c.creator_id
      left join comment_like cl on c.id = cl.comment_id
      group by u.id
    ) cd 
    where ua.person_id = OLD.creator_id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_comment_count
after insert or delete on comment
for each row
execute procedure person_aggregates_comment_count();

-- comment score
create function person_aggregates_comment_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update person_aggregates ua
    set comment_score = comment_score + NEW.score
    from comment c
    where ua.person_id = c.creator_id and c.id = NEW.comment_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates ua
    set comment_score = comment_score - OLD.score
    from comment c
    where ua.person_id = c.creator_id and c.id = OLD.comment_id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_comment_score
after insert or delete on comment_like
for each row
execute procedure person_aggregates_comment_score();

-- person_mention
alter table user_mention rename to person_mention;
alter sequence user_mention_id_seq rename to person_mention_id_seq;
alter index user_mention_pkey rename to person_mention_pkey;
alter index user_mention_recipient_id_comment_id_key rename to person_mention_recipient_id_comment_id_key;
alter table person_mention rename constraint user_mention_comment_id_fkey to person_mention_comment_id_fkey;
alter table person_mention rename constraint user_mention_recipient_id_fkey to person_mention_recipient_id_fkey;

-- user_ban
alter table user_ban rename to person_ban;
alter sequence user_ban_id_seq rename to person_ban_id_seq;
alter index user_ban_pkey rename to person_ban_pkey;
alter index user_ban_user_id_key rename to person_ban_person_id_key;
alter table person_ban rename constraint user_ban_user_id_fkey to person_ban_person_id_fkey;


