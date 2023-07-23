-- to ensure no confusion with all the migrations
--    this was mostly created by dumping from PostgreSQL 15.3 schema
-- note: hard-coded site_id = 1 is etablished convention in Lemmy's Rust code for local site.


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

