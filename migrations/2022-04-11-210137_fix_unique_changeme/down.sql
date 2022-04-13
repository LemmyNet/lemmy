create or replace function generate_unique_changeme() 
returns text language sql 
as $$
  select 'http://changeme_' || string_agg (substr('abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789', ceil (random() * 62)::integer, 1), '')
  from generate_series(1, 20)
$$;
