create or replace function person_aggregates_comment_count()
    returns trigger language plpgsql
as $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set comment_count = comment_count + 1 where person_id = NEW.creator_id;
    ELSIF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set comment_count = comment_count - 1 where person_id = OLD.creator_id;
    END IF;
    return null;
end $$;

create or replace function person_aggregates_post_count()
    returns trigger language plpgsql
as $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set post_count = post_count + 1 where person_id = NEW.creator_id;

    ELSIF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set post_count = post_count - 1 where person_id = OLD.creator_id;
    END IF;
    return null;
end $$;

create or replace function community_aggregates_comment_count()
    returns trigger language plpgsql
as $$
begin
  IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
update community_aggregates ca
set comments = comments + 1 from post p
where p.id = NEW.post_id
  and ca.community_id = p.community_id;
ELSIF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
update community_aggregates ca
set comments = comments - 1 from post p
where p.id = OLD.post_id
  and ca.community_id = p.community_id;

END IF;
return null;
end $$;