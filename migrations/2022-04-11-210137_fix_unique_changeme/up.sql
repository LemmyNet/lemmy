create or replace function generate_unique_changeme() 
returns text language sql 
as $$
  select 'http://changeme.ml/' || substr(md5(random()::text), 0, 25);
$$;
