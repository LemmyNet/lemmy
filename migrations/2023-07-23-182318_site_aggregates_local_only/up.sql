-- to ensure no confusion with all the migrations
--    this was mostly created by dumping from PostgreSQL 15.3 schema
-- note: hard-coded site_id = 1 is etablished convention in Lemmy's Rust code for local site.

create unique index idx_site_aggregates_site_id on site_aggregates (site_id);

CREATE OR REPLACE FUNCTION  site_aggregates_comment_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update site_aggregates
        set comments = comments - 1
        where site_id = 1;
    END IF;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_comment_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update site_aggregates
        set comments = comments + 1
        where site_id = 1;
    END IF;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_community_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update site_aggregates
        set communities = communities - 1
        where site_id = 1;
    END IF;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update site_aggregates
        set communities = communities + 1
        where site_id = 1;
    END IF;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_person_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    update site_aggregates
    set users = users - 1
    where site_id = 1;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_person_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    update site_aggregates 
    set users = users + 1
    where site_id = 1;
  return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_post_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update site_aggregates
        set posts = posts - 1
        where site_id = 1;
    END IF;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update site_aggregates
        set posts = posts + 1
        where site_id = 1;
    END IF;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  site_aggregates_activity(i text) RETURNS integer
    LANGUAGE plpgsql
    AS $$
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
    and u.bot_account = false
    union
    select p.creator_id from post p
    inner join person u on p.creator_id = u.id
    where p.published > ('now'::timestamp - i::interval)
    and u.local = true
    and u.bot_account = false
  ) a;
  return count_;
end;
$$;
