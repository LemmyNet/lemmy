-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION was_removed_or_deleted (TG_OP text, OLD record, NEW record)
    RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        RETURN FALSE;
    END IF;
    IF (TG_OP = 'DELETE') THEN
        RETURN TRUE;
    END IF;
    RETURN TG_OP = 'UPDATE'
        AND ((OLD.deleted = 'f'
                AND NEW.deleted = 't')
            OR (OLD.removed = 'f'
                AND NEW.removed = 't'));
END
$$;

