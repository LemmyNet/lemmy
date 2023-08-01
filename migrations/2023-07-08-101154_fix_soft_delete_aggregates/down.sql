-- 2023-06-19-120700_no_double_deletion/up.sql
create or replace function was_removed_or_deleted(TG_OP text, OLD record, NEW record)
RETURNS boolean
LANGUAGE plpgsql
as $$
    begin
        IF (TG_OP = 'INSERT') THEN
            return false;
        end if;

        IF (TG_OP = 'DELETE' AND OLD.deleted = 'f' AND OLD.removed = 'f') THEN
            return true;
        end if;

    return TG_OP = 'UPDATE' AND (
            (OLD.deleted = 'f' AND NEW.deleted = 't') OR
            (OLD.removed = 'f' AND NEW.removed = 't')
            );
END $$;

-- 2022-04-04-183652_update_community_aggregates_on_soft_delete/up.sql
create or replace function was_restored_or_created(TG_OP text, OLD record, NEW record)
    RETURNS boolean
    LANGUAGE plpgsql
as $$
begin
    IF (TG_OP = 'DELETE') THEN
        return false;
    end if;

    IF (TG_OP = 'INSERT') THEN
        return true;
    end if;

   return TG_OP = 'UPDATE' AND (
        (OLD.deleted = 't' AND NEW.deleted = 'f') OR
        (OLD.removed = 't' AND NEW.removed = 'f')
        );
END $$;

-- 2021-08-02-002342_comment_count_fixes/up.sql
create or replace function post_aggregates_comment_deleted()
returns trigger language plpgsql
as $$
begin
  IF NEW.deleted = TRUE THEN
    update post_aggregates pa
    set comments = comments - 1
    where pa.post_id = NEW.post_id;
  ELSE
    update post_aggregates pa
    set comments = comments + 1
    where pa.post_id = NEW.post_id;
  END IF;
  return null;
end $$;

create trigger post_aggregates_comment_set_deleted
after update of deleted on comment
for each row
execute procedure post_aggregates_comment_deleted();

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
    from post p
    where pa.post_id = p.id
    and pa.post_id = NEW.post_id
    -- Fix issue with being able to necro-bump your own post
    and NEW.creator_id != p.creator_id
    and pa.published > ('now'::timestamp - '2 days'::interval);

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  END IF;
  return null;
end $$;

-- 2020-12-10-152350_create_post_aggregates/up.sql
create or replace trigger post_aggregates_comment_count
after insert or delete on comment
for each row
execute procedure post_aggregates_comment_count();
