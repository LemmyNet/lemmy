-- First rename current newest comment time to newest_comment_time_necro
-- necro means that time is limited to 2 days, whereas newest_comment_time ignores that.
alter table post_aggregates rename column newest_comment_time to newest_comment_time_necro;

-- Add the newest_comment_time column
alter table post_aggregates add column newest_comment_time timestamp not null default now();

-- Set the current newest_comment_time based on the old ones
update post_aggregates set newest_comment_time = newest_comment_time_necro;

-- Add the indexes for this new column
create index idx_post_aggregates_newest_comment_time on post_aggregates (newest_comment_time desc);
create index idx_post_aggregates_stickied_newest_comment_time on post_aggregates (stickied desc, newest_comment_time desc);

-- Forgot to add index w/ stickied first for most comments:
create index idx_post_aggregates_stickied_comments on post_aggregates (stickied desc, comments desc);

-- Alter the comment trigger to set the newest_comment_time, and newest_comment_time_necro
create or replace function post_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set comments = comments + 1,
    newest_comment_time = NEW.published
    where pa.post_id = NEW.post_id;

    -- A 2 day necro-bump limit
    update post_aggregates pa
    set newest_comment_time_necro = NEW.published
    where pa.post_id = NEW.post_id
    and published > ('now'::timestamp - '2 days'::interval);
  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  END IF;
  return null;
end $$;
