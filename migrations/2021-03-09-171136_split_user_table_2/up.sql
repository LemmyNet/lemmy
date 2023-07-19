-- Person
-- Drop the 2 views user_alias_1, user_alias_2
drop view user_alias_1, user_alias_2;

-- rename the user_ table to person
alter table user_ rename to person;
alter sequence user__id_seq rename to person_id_seq;

-- create a new table local_user
create table local_user (
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  password_encrypted text not null,
  email text unique,
  admin boolean default false not null,
  show_nsfw boolean default false not null,
  theme character varying(20) default 'darkly'::character varying not null,
  default_sort_type smallint default 0 not null,
  default_listing_type smallint default 1 not null,
  lang character varying(20) default 'browser'::character varying not null,
  show_avatars boolean default true not null,
  send_notifications_to_email boolean default false not null,
  matrix_user_id text,
  unique (person_id)
);

-- Copy the local users over to the new table
insert into local_user 
(
  person_id,
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
alter table person_ban rename column user_id to person_id;
alter table person_ban rename constraint user_ban_user_id_fkey to person_ban_person_id_fkey;

-- comment_like
alter table comment_like rename column user_id to person_id;
alter index idx_comment_like_user rename to idx_comment_like_person;
alter table comment_like rename constraint comment_like_comment_id_user_id_key to comment_like_comment_id_person_id_key;
alter table comment_like rename constraint comment_like_user_id_fkey to comment_like_person_id_fkey;

-- comment_saved
alter table comment_saved rename column user_id to person_id;
alter table comment_saved rename constraint comment_saved_comment_id_user_id_key to comment_saved_comment_id_person_id_key;
alter table comment_saved rename constraint comment_saved_user_id_fkey to comment_saved_person_id_fkey;

-- community_follower
alter table community_follower rename column user_id to person_id;
alter table community_follower rename constraint community_follower_community_id_user_id_key to community_follower_community_id_person_id_key;
alter table community_follower rename constraint community_follower_user_id_fkey to community_follower_person_id_fkey;

-- community_moderator
alter table community_moderator rename column user_id to person_id;
alter table community_moderator rename constraint community_moderator_community_id_user_id_key to community_moderator_community_id_person_id_key;
alter table community_moderator rename constraint community_moderator_user_id_fkey to community_moderator_person_id_fkey;

-- community_user_ban
alter table community_user_ban rename to community_person_ban;
alter sequence community_user_ban_id_seq rename to community_person_ban_id_seq;
alter table community_person_ban rename column user_id to person_id;
alter table community_person_ban rename constraint community_user_ban_pkey to community_person_ban_pkey; 
alter table community_person_ban rename constraint community_user_ban_community_id_fkey to community_person_ban_community_id_fkey;
alter table community_person_ban rename constraint community_user_ban_community_id_user_id_key to community_person_ban_community_id_person_id_key;
alter table community_person_ban rename constraint community_user_ban_user_id_fkey to community_person_ban_person_id_fkey;

-- mod_add
alter table mod_add rename column mod_user_id to mod_person_id; 
alter table mod_add rename column other_user_id to other_person_id; 
alter table mod_add rename constraint mod_add_mod_user_id_fkey to mod_add_mod_person_id_fkey;
alter table mod_add rename constraint mod_add_other_user_id_fkey to mod_add_other_person_id_fkey;

-- mod_add_community
alter table mod_add_community rename column mod_user_id to mod_person_id; 
alter table mod_add_community rename column other_user_id to other_person_id; 
alter table mod_add_community rename constraint mod_add_community_mod_user_id_fkey to mod_add_community_mod_person_id_fkey;
alter table mod_add_community rename constraint mod_add_community_other_user_id_fkey to mod_add_community_other_person_id_fkey;

-- mod_ban
alter table mod_ban rename column mod_user_id to mod_person_id; 
alter table mod_ban rename column other_user_id to other_person_id; 
alter table mod_ban rename constraint mod_ban_mod_user_id_fkey to mod_ban_mod_person_id_fkey;
alter table mod_ban rename constraint mod_ban_other_user_id_fkey to mod_ban_other_person_id_fkey;

-- mod_ban_community
alter table mod_ban_from_community rename column mod_user_id to mod_person_id; 
alter table mod_ban_from_community rename column other_user_id to other_person_id; 
alter table mod_ban_from_community rename constraint mod_ban_from_community_mod_user_id_fkey to mod_ban_from_community_mod_person_id_fkey;
alter table mod_ban_from_community rename constraint mod_ban_from_community_other_user_id_fkey to mod_ban_from_community_other_person_id_fkey;

-- mod_lock_post
alter table mod_lock_post rename column mod_user_id to mod_person_id; 
alter table mod_lock_post rename constraint mod_lock_post_mod_user_id_fkey to mod_lock_post_mod_person_id_fkey;

-- mod_remove_comment
alter table mod_remove_comment rename column mod_user_id to mod_person_id; 
alter table mod_remove_comment rename constraint mod_remove_comment_mod_user_id_fkey to mod_remove_comment_mod_person_id_fkey;

-- mod_remove_community
alter table mod_remove_community rename column mod_user_id to mod_person_id; 
alter table mod_remove_community rename constraint mod_remove_community_mod_user_id_fkey to mod_remove_community_mod_person_id_fkey;

-- mod_remove_post
alter table mod_remove_post rename column mod_user_id to mod_person_id; 
alter table mod_remove_post rename constraint mod_remove_post_mod_user_id_fkey to mod_remove_post_mod_person_id_fkey;

-- mod_sticky_post
alter table mod_sticky_post rename column mod_user_id to mod_person_id; 
alter table mod_sticky_post rename constraint mod_sticky_post_mod_user_id_fkey to mod_sticky_post_mod_person_id_fkey;

-- password_reset_request
delete from password_reset_request;
alter table password_reset_request drop column user_id;
alter table password_reset_request add column local_user_id integer not null references local_user(id) on update cascade on delete cascade;

-- post_like
alter table post_like rename column user_id to person_id;
alter index idx_post_like_user rename to idx_post_like_person;
alter table post_like rename constraint post_like_post_id_user_id_key to post_like_post_id_person_id_key;
alter table post_like rename constraint post_like_user_id_fkey to post_like_person_id_fkey;

-- post_read
alter table post_read rename column user_id to person_id;
alter table post_read rename constraint post_read_post_id_user_id_key to post_read_post_id_person_id_key;
alter table post_read rename constraint post_read_user_id_fkey to post_read_person_id_fkey;

-- post_saved
alter table post_saved rename column user_id to person_id;
alter table post_saved rename constraint post_saved_post_id_user_id_key to post_saved_post_id_person_id_key;
alter table post_saved rename constraint post_saved_user_id_fkey to post_saved_person_id_fkey;

-- redo site aggregates trigger
create or replace function site_aggregates_activity(i text) returns integer
    language plpgsql
    as $$
declare
   count_ integer;
begin
  select count(*)
  into count_
  from (
    select c.creator_id from comment c
    inner join person u on c.creator_id = u.id
    where c.published > ('now'::timestamp - i::interval) 
    and u.local = true
    union
    select p.creator_id from post p
    inner join person u on p.creator_id = u.id
    where p.published > ('now'::timestamp - i::interval)
    and u.local = true
  ) a;
  return count_;
end;
$$;
