
DROP TRIGGER IF EXISTS post_aggregates_featured_local ON post;
DROP TRIGGER IF EXISTS post_aggregates_featured_community ON post;
drop function post_aggregates_featured_community;
drop function post_aggregates_featured_local;


alter table post ADD stickied boolean NOT NULL DEFAULT false;
Update post
set stickied = featured_community;
alter table post DROP COLUMN featured_community;
alter table post DROP COLUMN featured_local;

alter table post_aggregates ADD stickied boolean NOT NULL DEFAULT false;
Update post_aggregates
set stickied = featured_community;
alter table post_aggregates DROP COLUMN featured_community;
alter table post_aggregates DROP COLUMN featured_local;

alter table mod_feature_post
rename column featured TO stickied;

alter table mod_feature_post
DROP COLUMN is_featured_community;

alter table mod_feature_post
alter column stickied DROP NOT NULL;

alter table mod_feature_post
Rename To mod_sticky_post;

create function post_aggregates_stickied()
returns trigger language plpgsql
as $$
begin
  update post_aggregates pa
  set stickied = NEW.stickied
  where pa.post_id = NEW.id;

  return null;
end $$;

create trigger post_aggregates_stickied
after update on post
for each row
when (OLD.stickied is distinct from NEW.stickied)
execute procedure post_aggregates_stickied();