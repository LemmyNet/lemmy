-- This file undoes what is in `up.sql`
-- to ensure no confusion with all the migrations
--    this was mostly created by dumping from PostgreSQL 15.3 schema

DROP INDEX idx_site_aggregates_site_id;

CREATE OR REPLACE FUNCTION  site_aggregates_comment_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update site_aggregates sa
        set comments = comments - 1
        from site s
        where sa.site_id = s.id;
    END IF;
    return null;
end $$;



CREATE OR REPLACE FUNCTION  site_aggregates_comment_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update site_aggregates sa
        set comments = comments + 1
        from site s
        where sa.site_id = s.id;
    END IF;
    return null;
end $$;



CREATE OR REPLACE FUNCTION  site_aggregates_community_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
        IF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update site_aggregates sa
        set communities = communities - 1
        from site s
        where sa.site_id = s.id;
    END IF;
    return null;
end $$;



CREATE OR REPLACE FUNCTION  site_aggregates_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update site_aggregates sa
        set communities = communities + 1
        from site s
        where sa.site_id = s.id;
    END IF;
    return null;
end $$;



CREATE OR REPLACE FUNCTION  site_aggregates_person_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  -- Join to site since the creator might not be there anymore
  update site_aggregates sa
  set users = users - 1
  from site s
  where sa.site_id = s.id;
  return null;
end $$;



CREATE OR REPLACE FUNCTION  site_aggregates_person_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  update site_aggregates 
  set users = users + 1;
  return null;
end $$;



CREATE OR REPLACE FUNCTION  site_aggregates_post_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update site_aggregates sa
        set posts = posts - 1
        from site s
        where sa.site_id = s.id;
    END IF;
    return null;
end $$;



CREATE OR REPLACE FUNCTION  site_aggregates_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update site_aggregates sa
        set posts = posts + 1
        from site s
        where sa.site_id = s.id;
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
    inner join person pe on c.creator_id = pe.id
    where c.published > ('now'::timestamp - i::interval) 
    and u.local = true
    and pe.bot_account = false
    union
    select p.creator_id from post p
    inner join person u on p.creator_id = u.id
    inner join person pe on p.creator_id = pe.id
    where p.published > ('now'::timestamp - i::interval)
    and u.local = true
    and pe.bot_account = false
  ) a;
  return count_;
end;
$$;
