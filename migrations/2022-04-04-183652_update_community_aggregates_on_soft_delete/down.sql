drop trigger if exists community_aggregates_post_count on post;
drop trigger if exists community_aggregates_comment_count on comment;

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

create trigger community_aggregates_post_count
    after insert or delete on post
for each row
execute procedure community_aggregates_post_count();

-- comment count
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

create trigger community_aggregates_comment_count
    after insert or delete on comment
for each row
execute procedure community_aggregates_comment_count();
