create schema utils;

create table utils.deps_saved_ddl
(
  id serial NOT NULL,
  view_schema character varying(255),
  view_name character varying(255),
  ddl_to_run text,
  CONSTRAINT deps_saved_ddl_pkey PRIMARY KEY (id)
);

create or replace function utils.save_and_drop_views(p_view_schema name, p_view_name name)
    RETURNS void
    LANGUAGE plpgsql
    COST 100
AS $BODY$

declare
  v_curr record;
begin
for v_curr in 
(
  select obj_schema, obj_name, obj_type from
  (
  with recursive recursive_deps(obj_schema, obj_name, obj_type, depth) as 
  (
    select p_view_schema::name, p_view_name, null::varchar, 0
    union
    select dep_schema::varchar, dep_name::varchar, dep_type::varchar, recursive_deps.depth + 1 from 
    (
      select ref_nsp.nspname ref_schema, ref_cl.relname ref_name, 
      rwr_cl.relkind dep_type,
      rwr_nsp.nspname dep_schema,
      rwr_cl.relname dep_name
      from pg_depend dep
      join pg_class ref_cl on dep.refobjid = ref_cl.oid
      join pg_namespace ref_nsp on ref_cl.relnamespace = ref_nsp.oid
      join pg_rewrite rwr on dep.objid = rwr.oid
      join pg_class rwr_cl on rwr.ev_class = rwr_cl.oid
      join pg_namespace rwr_nsp on rwr_cl.relnamespace = rwr_nsp.oid
      where dep.deptype = 'n'
      and dep.classid = 'pg_rewrite'::regclass
    ) deps
    join recursive_deps on deps.ref_schema = recursive_deps.obj_schema and deps.ref_name = recursive_deps.obj_name
    where (deps.ref_schema != deps.dep_schema or deps.ref_name != deps.dep_name)
  )
  select obj_schema, obj_name, obj_type, depth
  from recursive_deps 
  where depth > 0
  ) t
  group by obj_schema, obj_name, obj_type
  order by max(depth) desc
) loop
  if v_curr.obj_type = 'v' then
    insert into utils.deps_saved_ddl(view_schema, view_name, ddl_to_run)
    select p_view_schema, p_view_name, 'CREATE VIEW ' || v_curr.obj_schema || '.' || v_curr.obj_name || ' AS ' || view_definition
    from information_schema.views
    where table_schema = v_curr.obj_schema and table_name = v_curr.obj_name;

    execute 'DROP VIEW' || ' ' || v_curr.obj_schema || '.' || v_curr.obj_name;
  end if;
end loop;
end;
$BODY$;

create or replace function utils.restore_views(p_view_schema character varying, p_view_name character varying)
  RETURNS void 
  LANGUAGE plpgsql
  COST 100
AS $BODY$
declare
  v_curr record;
begin
for v_curr in 
(
  select ddl_to_run, id 
  from utils.deps_saved_ddl
  where view_schema = p_view_schema and view_name = p_view_name
  order by id desc
) loop
begin
  execute v_curr.ddl_to_run;
  delete from utils.deps_saved_ddl where id = v_curr.id;
  EXCEPTION WHEN OTHERS THEN
      -- keep looping, but please check for errors or remove left overs to handle manually
	  end;
end loop;
end;
$BODY$;