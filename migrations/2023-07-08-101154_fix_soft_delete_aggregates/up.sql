-- Fix for duplicated decrementations when both `deleted` and `removed` fields are set subsequently
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

    return TG_OP = 'UPDATE' AND OLD.deleted = 'f' AND OLD.removed = 'f' AND (
            NEW.deleted = 't' OR NEW.removed = 't'
            );
END $$;

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

   return TG_OP = 'UPDATE' AND NEW.deleted = 'f' AND NEW.removed = 'f' AND (
            OLD.deleted = 't' OR OLD.removed = 't'
            );
END $$;

-- Fix for post's comment count not updating after setting `removed` to 't'
drop trigger if exists post_aggregates_comment_set_deleted on comment;
drop function post_aggregates_comment_deleted();

create or replace function post_aggregates_comment_count()
    returns trigger language plpgsql
as $$
begin
    -- Check for post existence - it may not exist anymore
    IF TG_OP = 'INSERT' OR EXISTS (
        select 1 from post p where p.id = OLD.post_id
    ) THEN
        IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
            update post_aggregates pa
            set comments = comments + 1 where pa.post_id = NEW.post_id;
        ELSIF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
            update post_aggregates pa
            set comments = comments - 1 where pa.post_id = OLD.post_id;
        END IF;
    END IF;

    IF TG_OP = 'INSERT' THEN
        update post_aggregates pa
        set newest_comment_time = NEW.published
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
    END IF;

    return null;
end $$;

create or replace trigger post_aggregates_comment_count
    after insert or delete or update of removed, deleted on comment
    for each row
execute procedure post_aggregates_comment_count();