create or replace function generate_unique_changeme() 
returns text language sql 
as $$
  select 'http://changeme_' || substr(md5(random()::text), 0, 25) || '.ml';
$$;
