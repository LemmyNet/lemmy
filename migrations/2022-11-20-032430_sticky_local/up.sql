
DROP TRIGGER IF EXISTS post_aggregates_stickied ON post;
drop function 
  post_aggregates_stickied;


alter table post ADD featured_community boolean NOT NULL DEFAULT false;
alter table post ADD featured_local boolean NOT NULL DEFAULT false;
update post
set featured_community = stickied;
alter table post DROP COLUMN stickied;

alter table post_aggregates ADD featured_community boolean NOT NULL DEFAULT false;
alter table post_aggregates ADD featured_local boolean NOT NULL DEFAULT false;
update post_aggregates
set featured_community = stickied;
alter table post_aggregates DROP COLUMN stickied;

alter table mod_sticky_post
rename column stickied TO featured;

alter table mod_sticky_post
alter column featured SET NOT NULL;

alter table mod_sticky_post
ADD is_featured_community boolean NOT NULL DEFAULT true;

alter table mod_sticky_post
Rename To mod_feature_post;

create function post_aggregates_featured_community()
returns trigger language plpgsql
as $$
begin
  update post_aggregates pa
  set featured_community = NEW.featured_community
  where pa.post_id = NEW.id;
  return null;
end $$;

create function post_aggregates_featured_local()
returns trigger language plpgsql
as $$
begin
  update post_aggregates pa
  set featured_local = NEW.featured_local
  where pa.post_id = NEW.id;
  return null;
end $$;

CREATE TRIGGER post_aggregates_featured_community
    AFTER UPDATE 
    ON public.post
    FOR EACH ROW
    WHEN (old.featured_community IS DISTINCT FROM new.featured_community)
    EXECUTE FUNCTION public.post_aggregates_featured_community();

CREATE TRIGGER post_aggregates_featured_local
    AFTER UPDATE 
    ON public.post
    FOR EACH ROW
    WHEN (old.featured_local IS DISTINCT FROM new.featured_local)
    EXECUTE FUNCTION public.post_aggregates_featured_local();