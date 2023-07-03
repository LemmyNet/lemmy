-- This file should undo anything in `up.sql`
create or replace function was_removed_or_deleted(TG_OP text, OLD record, NEW record)
RETURNS boolean
LANGUAGE plpgsql
as $$
    begin
        IF (TG_OP = 'INSERT') THEN
            return false;
        end if;

        IF (TG_OP = 'DELETE') THEN
            return true;
        end if;

    return TG_OP = 'UPDATE' AND (
            (OLD.deleted = 'f' AND NEW.deleted = 't') OR
            (OLD.removed = 'f' AND NEW.removed = 't')
            );
END $$;
