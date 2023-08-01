create or replace function site_aggregates_site()
returns trigger language plpgsql
as $$
begin
  -- we only ever want to have a single value in site_aggregate because the site_aggregate triggers update all rows in that table.
  -- a cleaner check would be to insert it for the local_site but that would break assumptions at least in the tests
  IF (TG_OP = 'INSERT') AND NOT EXISTS (select id from site_aggregates limit 1) THEN
    insert into site_aggregates (site_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from site_aggregates where site_id = OLD.id;
  END IF;
  return null;
end $$;

delete from site_aggregates a where not exists (select id from local_site s where s.site_id = a.site_id);