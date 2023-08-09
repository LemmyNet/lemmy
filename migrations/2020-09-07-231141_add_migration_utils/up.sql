CREATE SCHEMA utils;

CREATE TABLE utils.deps_saved_ddl (
    id serial NOT NULL,
    view_schema character varying(255),
    view_name character varying(255),
    ddl_to_run text,
    CONSTRAINT deps_saved_ddl_pkey PRIMARY KEY (id)
);

CREATE OR REPLACE FUNCTION utils.save_and_drop_views (p_view_schema name, p_view_name name)
    RETURNS void
    LANGUAGE plpgsql
    COST 100
    AS $BODY$
DECLARE
    v_curr record;
BEGIN
    FOR v_curr IN (
        SELECT
            obj_schema,
            obj_name,
            obj_type
        FROM ( WITH RECURSIVE recursive_deps (
                obj_schema,
                obj_name,
                obj_type,
                depth
) AS (
                SELECT
                    p_view_schema::name,
                    p_view_name,
                    NULL::varchar,
                    0
                UNION
                SELECT
                    dep_schema::varchar,
                    dep_name::varchar,
                    dep_type::varchar,
                    recursive_deps.depth + 1
                FROM (
                    SELECT
                        ref_nsp.nspname ref_schema,
                        ref_cl.relname ref_name,
                        rwr_cl.relkind dep_type,
                        rwr_nsp.nspname dep_schema,
                        rwr_cl.relname dep_name
                    FROM
                        pg_depend dep
                        JOIN pg_class ref_cl ON dep.refobjid = ref_cl.oid
                        JOIN pg_namespace ref_nsp ON ref_cl.relnamespace = ref_nsp.oid
                        JOIN pg_rewrite rwr ON dep.objid = rwr.oid
                        JOIN pg_class rwr_cl ON rwr.ev_class = rwr_cl.oid
                        JOIN pg_namespace rwr_nsp ON rwr_cl.relnamespace = rwr_nsp.oid
                    WHERE
                        dep.deptype = 'n'
                        AND dep.classid = 'pg_rewrite'::regclass) deps
                    JOIN recursive_deps ON deps.ref_schema = recursive_deps.obj_schema
                        AND deps.ref_name = recursive_deps.obj_name
                WHERE (deps.ref_schema != deps.dep_schema
                    OR deps.ref_name != deps.dep_name))
            SELECT
                obj_schema,
                obj_name,
                obj_type,
                depth
            FROM
                recursive_deps
            WHERE
                depth > 0) t
        GROUP BY
            obj_schema,
            obj_name,
            obj_type
        ORDER BY
            max(depth) DESC)
            LOOP
                IF v_curr.obj_type = 'v' THEN
                    INSERT INTO utils.deps_saved_ddl (view_schema, view_name, ddl_to_run)
                    SELECT
                        p_view_schema,
                        p_view_name,
                        'CREATE VIEW ' || v_curr.obj_schema || '.' || v_curr.obj_name || ' AS ' || view_definition
                    FROM
                        information_schema.views
                    WHERE
                        table_schema = v_curr.obj_schema
                        AND table_name = v_curr.obj_name;
                    EXECUTE 'DROP VIEW' || ' ' || v_curr.obj_schema || '.' || v_curr.obj_name;
                END IF;
            END LOOP;
END;
$BODY$;

CREATE OR REPLACE FUNCTION utils.restore_views (p_view_schema character varying, p_view_name character varying)
    RETURNS void
    LANGUAGE plpgsql
    COST 100
    AS $BODY$
DECLARE
    v_curr record;
BEGIN
    FOR v_curr IN (
        SELECT
            ddl_to_run,
            id
        FROM
            utils.deps_saved_ddl
        WHERE
            view_schema = p_view_schema
            AND view_name = p_view_name
        ORDER BY
            id DESC)
            LOOP
                BEGIN
                    EXECUTE v_curr.ddl_to_run;
                    DELETE FROM utils.deps_saved_ddl
                    WHERE id = v_curr.id;
                EXCEPTION
                    WHEN OTHERS THEN
                        -- keep looping, but please check for errors or remove left overs to handle manually
                END;
    END LOOP;
END;

$BODY$;

