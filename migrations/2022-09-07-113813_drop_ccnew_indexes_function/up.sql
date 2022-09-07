CREATE OR REPLACE FUNCTION drop_ccnew_indexes() RETURNS INTEGER AS $$
DECLARE
i RECORD;
BEGIN
  FOR i IN
    (SELECT relname FROM pg_class WHERE relname like '%ccnew%')
    LOOP
      EXECUTE 'DROP INDEX ' || i.relname;
    END LOOP;
    RETURN 1;
  END;
$$ LANGUAGE plpgsql;

