drop trigger if exists community_aggregates_post_count on post;
drop trigger if exists community_aggregates_comment_count on comment;
drop trigger if exists site_aggregates_comment_insert on comment;
drop trigger if exists site_aggregates_comment_delete on comment;
drop trigger if exists site_aggregates_post_insert on post;
drop trigger if exists site_aggregates_post_delete on post;
drop trigger if exists site_aggregates_community_insert on community;
drop trigger if exists site_aggregates_community_delete on community;
drop trigger if exists person_aggregates_post_count on post;
drop trigger if exists person_aggregates_comment_count on comment;

drop function was_removed_or_deleted(TG_OP text, OLD record, NEW record);
drop function was_restored_or_created(TG_OP text, OLD record, NEW record);

-- Community aggregate functions

create or replace function community_aggregates_post_count()
    returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
update community_aggregates
set posts = posts + 1 where community_id = NEW.community_id;
ELSIF (TG_OP = 'DELETE') THEN
update community_aggregates
set posts = posts - 1 where community_id = OLD.community_id;

-- Update the counts if the post got deleted
update community_aggregates ca
set posts = coalesce(cd.posts, 0),
    comments = coalesce(cd.comments, 0)
    from ( 
      select 
      c.id,
      count(distinct p.id) as posts,
      count(distinct ct.id) as comments
      from community c
      left join post p on c.id = p.community_id
      left join comment ct on p.id = ct.post_id
      group by c.id
    ) cd
where ca.community_id = OLD.community_id;
END IF;
return null;
end $$;

create or replace function community_aggregates_comment_count()
    returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
update community_aggregates ca
set comments = comments + 1 from comment c, post p
where p.id = c.post_id
  and p.id = NEW.post_id
  and ca.community_id = p.community_id;
ELSIF (TG_OP = 'DELETE') THEN
update community_aggregates ca
set comments = comments - 1 from comment c, post p
where p.id = c.post_id
  and p.id = OLD.post_id
  and ca.community_id = p.community_id;

END IF;
return null;
end $$;

-- Community aggregate triggers

create trigger community_aggregates_post_count
    after insert or delete on post
for each row
execute procedure community_aggregates_post_count();

create trigger community_aggregates_comment_count
    after insert or delete on comment
for each row
execute procedure community_aggregates_comment_count();

-- Site aggregate functions

create or replace function site_aggregates_post_insert()
    returns trigger language plpgsql
as $$
begin
update site_aggregates
set posts = posts + 1;
return null;
end $$;

create or replace function site_aggregates_post_delete()
    returns trigger language plpgsql
as $$
begin
update site_aggregates sa
set posts = posts - 1
    from site s
where sa.site_id = s.id;
return null;
end $$;

create or replace function site_aggregates_comment_insert()
    returns trigger language plpgsql
as $$
begin
update site_aggregates
set comments = comments + 1;
return null;
end $$;

create or replace function site_aggregates_comment_delete()
    returns trigger language plpgsql
as $$
begin
update site_aggregates sa
set comments = comments - 1
    from site s
where sa.site_id = s.id;
return null;
end $$;

create or replace function site_aggregates_community_insert()
    returns trigger language plpgsql
as $$
begin
update site_aggregates
set communities = communities + 1;
return null;
end $$;

create or replace function site_aggregates_community_delete()
    returns trigger language plpgsql
as $$
begin
update site_aggregates sa
set communities = communities - 1
    from site s
where sa.site_id = s.id;
return null;
end $$;


-- Site update triggers

create trigger site_aggregates_post_insert
    after insert on post
    for each row
    when (NEW.local = true)
    execute procedure site_aggregates_post_insert();

create trigger site_aggregates_post_delete
    after delete on post
    for each row
    when (OLD.local = true)
    execute procedure site_aggregates_post_delete();

create trigger site_aggregates_comment_insert
    after insert on comment
    for each row
    when (NEW.local = true)
    execute procedure site_aggregates_comment_insert();

create trigger site_aggregates_comment_delete
    after delete on comment
    for each row
    when (OLD.local = true)
    execute procedure site_aggregates_comment_delete();

create trigger site_aggregates_community_insert
    after insert on community
    for each row
    when (NEW.local = true)
    execute procedure site_aggregates_community_insert();

create trigger site_aggregates_community_delete
    after delete on community
    for each row
    when (OLD.local = true)
    execute procedure site_aggregates_community_delete();

-- Person aggregate functions

create or replace function person_aggregates_post_count()
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

create or replace function person_aggregates_comment_count()
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


-- Person aggregate triggers

create trigger person_aggregates_post_count
    after insert or delete on post
    for each row
execute procedure person_aggregates_post_count();

create trigger person_aggregates_comment_count
    after insert or delete on comment
    for each row
execute procedure person_aggregates_comment_count();