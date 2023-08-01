create or replace function site_aggregates_site()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into site_aggregates (site_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from site_aggregates where site_id = OLD.id;
  END IF;
  return null;
end $$;