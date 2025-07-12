--
-- PostgreSQL database dump
--

-- Dumped from database version 17.5
-- Dumped by pg_dump version 17.5

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: public; Type: SCHEMA; Schema: -; Owner: lemmy
--

-- *not* creating schema, since initdb creates it


ALTER SCHEMA public OWNER TO lemmy;

--
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: lemmy
--

COMMENT ON SCHEMA public IS '';


--
-- Name: r; Type: SCHEMA; Schema: -; Owner: lemmy
--

CREATE SCHEMA r;


ALTER SCHEMA r OWNER TO lemmy;

--
-- Name: utils; Type: SCHEMA; Schema: -; Owner: lemmy
--

CREATE SCHEMA utils;


ALTER SCHEMA utils OWNER TO lemmy;

--
-- Name: ltree; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS ltree WITH SCHEMA public;


--
-- Name: EXTENSION ltree; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION ltree IS 'data type for hierarchical tree-like structures';


--
-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;


--
-- Name: EXTENSION pg_trgm; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


--
-- Name: actor_type_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.actor_type_enum AS ENUM (
    'site',
    'community',
    'person'
);


ALTER TYPE public.actor_type_enum OWNER TO lemmy;

--
-- Name: comment_sort_type_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.comment_sort_type_enum AS ENUM (
    'Hot',
    'Top',
    'New',
    'Old',
    'Controversial'
);


ALTER TYPE public.comment_sort_type_enum OWNER TO lemmy;

--
-- Name: community_follower_state; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.community_follower_state AS ENUM (
    'Accepted',
    'Pending',
    'ApprovalRequired'
);


ALTER TYPE public.community_follower_state OWNER TO lemmy;

--
-- Name: community_visibility; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.community_visibility AS ENUM (
    'Public',
    'LocalOnlyPublic',
    'LocalOnlyPrivate',
    'Private',
    'Unlisted'
);


ALTER TYPE public.community_visibility OWNER TO lemmy;

--
-- Name: federation_mode_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.federation_mode_enum AS ENUM (
    'All',
    'Local',
    'Disable'
);


ALTER TYPE public.federation_mode_enum OWNER TO lemmy;

--
-- Name: listing_type_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.listing_type_enum AS ENUM (
    'All',
    'Local',
    'Subscribed',
    'ModeratorView',
    'Suggested'
);


ALTER TYPE public.listing_type_enum OWNER TO lemmy;

--
-- Name: post_listing_mode_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.post_listing_mode_enum AS ENUM (
    'List',
    'Card',
    'SmallCard'
);


ALTER TYPE public.post_listing_mode_enum OWNER TO lemmy;

--
-- Name: post_sort_type_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.post_sort_type_enum AS ENUM (
    'Active',
    'Hot',
    'New',
    'Old',
    'Top',
    'MostComments',
    'NewComments',
    'Controversial',
    'Scaled'
);


ALTER TYPE public.post_sort_type_enum OWNER TO lemmy;

--
-- Name: registration_mode_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.registration_mode_enum AS ENUM (
    'Closed',
    'RequireApplication',
    'Open'
);


ALTER TYPE public.registration_mode_enum OWNER TO lemmy;

--
-- Name: vote_show_enum; Type: TYPE; Schema: public; Owner: lemmy
--

CREATE TYPE public.vote_show_enum AS ENUM (
    'Show',
    'ShowForOthers',
    'Hide'
);


ALTER TYPE public.vote_show_enum OWNER TO lemmy;

--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: lemmy
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO lemmy;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: lemmy
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (NEW IS DISTINCT FROM OLD AND NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at) THEN
        NEW.updated_at := CURRENT_TIMESTAMP;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO lemmy;

--
-- Name: drop_ccnew_indexes(); Type: FUNCTION; Schema: public; Owner: lemmy
--

CREATE FUNCTION public.drop_ccnew_indexes() RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    i RECORD;
BEGIN
    FOR i IN (
        SELECT
            relname
        FROM
            pg_class
        WHERE
            relname LIKE '%ccnew%')
        LOOP
            EXECUTE 'DROP INDEX ' || i.relname;
        END LOOP;
    RETURN 1;
END;
$$;


ALTER FUNCTION public.drop_ccnew_indexes() OWNER TO lemmy;

--
-- Name: forbid_diesel_cli(); Type: FUNCTION; Schema: public; Owner: lemmy
--

CREATE FUNCTION public.forbid_diesel_cli() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NOT EXISTS (
        SELECT
        FROM
            pg_locks
        WHERE (locktype, pid, objid) = ('advisory', pg_backend_pid(), 0)) THEN
RAISE 'migrations must be managed using lemmy_server instead of diesel CLI';
END IF;
    RETURN NULL;
END;
$$;


ALTER FUNCTION public.forbid_diesel_cli() OWNER TO lemmy;

--
-- Name: generate_unique_changeme(); Type: FUNCTION; Schema: public; Owner: lemmy
--

CREATE FUNCTION public.generate_unique_changeme() RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://changeme.invalid/seq/' || nextval('changeme_seq')::text;
$$;


ALTER FUNCTION public.generate_unique_changeme() OWNER TO lemmy;

--
-- Name: random_smallint(); Type: FUNCTION; Schema: public; Owner: lemmy
--

CREATE FUNCTION public.random_smallint() RETURNS smallint
    LANGUAGE sql PARALLEL RESTRICTED
    RETURN trunc(((random() * (65536)::double precision) - (32768)::double precision));


ALTER FUNCTION public.random_smallint() OWNER TO lemmy;

--
-- Name: reverse_timestamp_sort(timestamp with time zone); Type: FUNCTION; Schema: public; Owner: lemmy
--

CREATE FUNCTION public.reverse_timestamp_sort(t timestamp with time zone) RETURNS bigint
    LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    RETURN (-1000000 * EXTRACT(EPOCH FROM t))::bigint;
END;
$$;


ALTER FUNCTION public.reverse_timestamp_sort(t timestamp with time zone) OWNER TO lemmy;

--
-- Name: comment_actions_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_actions_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                WITH comment_diff AS ( UPDATE
                        comment AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (comment_actions).comment_id, coalesce(sum(count_diff) FILTER (WHERE (comment_actions).like_score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (comment_actions).like_score != 1), 0) AS downvotes FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment_actions AS comment_actions
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment_actions AS comment_actions
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
                WHERE (comment_actions).like_score IS NOT NULL GROUP BY (comment_actions).comment_id) AS diff
            WHERE
                a.id = diff.comment_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    a.creator_id AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person AS a
            SET
                comment_score = a.comment_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM comment_diff GROUP BY creator_id) AS diff
                WHERE
                    a.id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$;


ALTER FUNCTION r.comment_actions_delete_statement() OWNER TO lemmy;

--
-- Name: comment_actions_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_actions_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                WITH comment_diff AS ( UPDATE
                        comment AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (comment_actions).comment_id, coalesce(sum(count_diff) FILTER (WHERE (comment_actions).like_score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (comment_actions).like_score != 1), 0) AS downvotes FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment_actions AS comment_actions
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment_actions AS comment_actions
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
                WHERE (comment_actions).like_score IS NOT NULL GROUP BY (comment_actions).comment_id) AS diff
            WHERE
                a.id = diff.comment_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    a.creator_id AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person AS a
            SET
                comment_score = a.comment_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM comment_diff GROUP BY creator_id) AS diff
                WHERE
                    a.id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$;


ALTER FUNCTION r.comment_actions_insert_statement() OWNER TO lemmy;

--
-- Name: comment_actions_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_actions_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                WITH comment_diff AS ( UPDATE
                        comment AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (comment_actions).comment_id, coalesce(sum(count_diff) FILTER (WHERE (comment_actions).like_score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (comment_actions).like_score != 1), 0) AS downvotes FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment_actions AS comment_actions
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment_actions AS comment_actions
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
                WHERE (comment_actions).like_score IS NOT NULL GROUP BY (comment_actions).comment_id) AS diff
            WHERE
                a.id = diff.comment_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    a.creator_id AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person AS a
            SET
                comment_score = a.comment_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM comment_diff GROUP BY creator_id) AS diff
                WHERE
                    a.id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$;


ALTER FUNCTION r.comment_actions_update_statement() OWNER TO lemmy;

--
-- Name: comment_change_values(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_change_values() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    id text = NEW.id::text;
BEGIN
    -- Make `path` end with `id` if it doesn't already
    IF NOT (NEW.path ~ ('*.' || id)::lquery) THEN
        NEW.path = NEW.path || id;
    END IF;
    -- Set local ap_id
    IF NEW.local THEN
        NEW.ap_id = coalesce(NEW.ap_id, r.local_url ('/comment/' || id));
    END IF;
    RETURN NEW;
END
$$;


ALTER FUNCTION r.comment_change_values() OWNER TO lemmy;

--
-- Name: comment_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Prevent infinite recursion
    IF (
        SELECT
            count(*)
    FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows) = 0 THEN
        RETURN NULL;
END IF;
UPDATE
    person AS a
SET
    comment_count = a.comment_count + diff.comment_count
FROM (
    SELECT
        (comment).creator_id,
        coalesce(sum(count_diff), 0) AS comment_count
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (comment)
    GROUP BY
        (comment).creator_id) AS diff
WHERE
    a.id = diff.creator_id
    AND diff.comment_count != 0;
UPDATE
    comment AS a
SET
    child_count = a.child_count + diff.child_count
FROM (
    SELECT
        parent_id,
        coalesce(sum(count_diff), 0) AS child_count
    FROM (
        -- For each inserted or deleted comment, this outputs 1 row for each parent comment.
        -- For example, this:
        --
        --  count_diff | (comment).path
        -- ------------+----------------
        --  1          | 0.5.6.7
        --  1          | 0.5.6.7.8
        --
        -- becomes this:
        --
        --  count_diff | parent_id
        -- ------------+-----------
        --  1          | 5
        --  1          | 6
        --  1          | 5
        --  1          | 6
        --  1          | 7
        SELECT
            count_diff,
            parent_id
        FROM
             (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows,
            LATERAL r.parent_comment_ids ((comment).path) AS parent_id) AS expanded_old_and_new_rows
    GROUP BY
        parent_id) AS diff
WHERE
    a.id = diff.parent_id
    AND diff.child_count != 0;
UPDATE
    post AS a
SET
    comments = a.comments + diff.comments,
    newest_comment_time_at = GREATEST (a.newest_comment_time_at, diff.newest_comment_time_at),
    newest_comment_time_necro_at = GREATEST (a.newest_comment_time_necro_at, diff.newest_comment_time_necro_at)
FROM (
    SELECT
        post.id AS post_id,
        coalesce(sum(count_diff), 0) AS comments,
        -- Old rows are excluded using `count_diff = 1`
        max((comment).published_at) FILTER (WHERE count_diff = 1) AS newest_comment_time_at,
        max((comment).published_at) FILTER (WHERE count_diff = 1
            -- Ignore comments from the post's creator
            AND post.creator_id != (comment).creator_id
        -- Ignore comments on old posts
        AND post.published_at > ((comment).published_at - '2 days'::interval)) AS newest_comment_time_necro_at
FROM
     (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
    LEFT JOIN post ON post.id = (comment).post_id
WHERE
    r.is_counted (comment)
GROUP BY
    post.id) AS diff
WHERE
    a.id = diff.post_id
    AND (diff.comments,
        GREATEST (a.newest_comment_time_at, diff.newest_comment_time_at),
        GREATEST (a.newest_comment_time_necro_at, diff.newest_comment_time_necro_at)) != (0,
        a.newest_comment_time_at,
        a.newest_comment_time_necro_at);
UPDATE
    local_site AS a
SET
    comments = a.comments + diff.comments
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS comments
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (comment)
        AND (comment).local) AS diff
WHERE
    diff.comments != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.comment_delete_statement() OWNER TO lemmy;

--
-- Name: comment_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Prevent infinite recursion
    IF (
        SELECT
            count(*)
    FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows) = 0 THEN
        RETURN NULL;
END IF;
UPDATE
    person AS a
SET
    comment_count = a.comment_count + diff.comment_count
FROM (
    SELECT
        (comment).creator_id,
        coalesce(sum(count_diff), 0) AS comment_count
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (comment)
    GROUP BY
        (comment).creator_id) AS diff
WHERE
    a.id = diff.creator_id
    AND diff.comment_count != 0;
UPDATE
    comment AS a
SET
    child_count = a.child_count + diff.child_count
FROM (
    SELECT
        parent_id,
        coalesce(sum(count_diff), 0) AS child_count
    FROM (
        -- For each inserted or deleted comment, this outputs 1 row for each parent comment.
        -- For example, this:
        --
        --  count_diff | (comment).path
        -- ------------+----------------
        --  1          | 0.5.6.7
        --  1          | 0.5.6.7.8
        --
        -- becomes this:
        --
        --  count_diff | parent_id
        -- ------------+-----------
        --  1          | 5
        --  1          | 6
        --  1          | 5
        --  1          | 6
        --  1          | 7
        SELECT
            count_diff,
            parent_id
        FROM
             (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows,
            LATERAL r.parent_comment_ids ((comment).path) AS parent_id) AS expanded_old_and_new_rows
    GROUP BY
        parent_id) AS diff
WHERE
    a.id = diff.parent_id
    AND diff.child_count != 0;
UPDATE
    post AS a
SET
    comments = a.comments + diff.comments,
    newest_comment_time_at = GREATEST (a.newest_comment_time_at, diff.newest_comment_time_at),
    newest_comment_time_necro_at = GREATEST (a.newest_comment_time_necro_at, diff.newest_comment_time_necro_at)
FROM (
    SELECT
        post.id AS post_id,
        coalesce(sum(count_diff), 0) AS comments,
        -- Old rows are excluded using `count_diff = 1`
        max((comment).published_at) FILTER (WHERE count_diff = 1) AS newest_comment_time_at,
        max((comment).published_at) FILTER (WHERE count_diff = 1
            -- Ignore comments from the post's creator
            AND post.creator_id != (comment).creator_id
        -- Ignore comments on old posts
        AND post.published_at > ((comment).published_at - '2 days'::interval)) AS newest_comment_time_necro_at
FROM
     (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    LEFT JOIN post ON post.id = (comment).post_id
WHERE
    r.is_counted (comment)
GROUP BY
    post.id) AS diff
WHERE
    a.id = diff.post_id
    AND (diff.comments,
        GREATEST (a.newest_comment_time_at, diff.newest_comment_time_at),
        GREATEST (a.newest_comment_time_necro_at, diff.newest_comment_time_necro_at)) != (0,
        a.newest_comment_time_at,
        a.newest_comment_time_necro_at);
UPDATE
    local_site AS a
SET
    comments = a.comments + diff.comments
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS comments
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (comment)
        AND (comment).local) AS diff
WHERE
    diff.comments != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.comment_insert_statement() OWNER TO lemmy;

--
-- Name: comment_report_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_report_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        comment AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (comment_report).comment_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (comment_report).resolved
                AND NOT (comment_report).violates_instance_rules), 0) AS unresolved_report_count
FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment_report AS comment_report
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment_report AS comment_report
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows GROUP BY (comment_report).comment_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
AND a.id = diff.comment_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.comment_report_delete_statement() OWNER TO lemmy;

--
-- Name: comment_report_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_report_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        comment AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (comment_report).comment_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (comment_report).resolved
                AND NOT (comment_report).violates_instance_rules), 0) AS unresolved_report_count
FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment_report AS comment_report
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment_report AS comment_report
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows GROUP BY (comment_report).comment_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
AND a.id = diff.comment_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.comment_report_insert_statement() OWNER TO lemmy;

--
-- Name: comment_report_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_report_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        comment AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (comment_report).comment_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (comment_report).resolved
                AND NOT (comment_report).violates_instance_rules), 0) AS unresolved_report_count
FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment_report AS comment_report
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment_report AS comment_report
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows GROUP BY (comment_report).comment_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
AND a.id = diff.comment_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.comment_report_update_statement() OWNER TO lemmy;

--
-- Name: comment_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.comment_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Prevent infinite recursion
    IF (
        SELECT
            count(*)
    FROM  (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows) = 0 THEN
        RETURN NULL;
END IF;
UPDATE
    person AS a
SET
    comment_count = a.comment_count + diff.comment_count
FROM (
    SELECT
        (comment).creator_id,
        coalesce(sum(count_diff), 0) AS comment_count
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (comment)
    GROUP BY
        (comment).creator_id) AS diff
WHERE
    a.id = diff.creator_id
    AND diff.comment_count != 0;
UPDATE
    comment AS a
SET
    child_count = a.child_count + diff.child_count
FROM (
    SELECT
        parent_id,
        coalesce(sum(count_diff), 0) AS child_count
    FROM (
        -- For each inserted or deleted comment, this outputs 1 row for each parent comment.
        -- For example, this:
        --
        --  count_diff | (comment).path
        -- ------------+----------------
        --  1          | 0.5.6.7
        --  1          | 0.5.6.7.8
        --
        -- becomes this:
        --
        --  count_diff | parent_id
        -- ------------+-----------
        --  1          | 5
        --  1          | 6
        --  1          | 5
        --  1          | 6
        --  1          | 7
        SELECT
            count_diff,
            parent_id
        FROM
             (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows,
            LATERAL r.parent_comment_ids ((comment).path) AS parent_id) AS expanded_old_and_new_rows
    GROUP BY
        parent_id) AS diff
WHERE
    a.id = diff.parent_id
    AND diff.child_count != 0;
UPDATE
    post AS a
SET
    comments = a.comments + diff.comments,
    newest_comment_time_at = GREATEST (a.newest_comment_time_at, diff.newest_comment_time_at),
    newest_comment_time_necro_at = GREATEST (a.newest_comment_time_necro_at, diff.newest_comment_time_necro_at)
FROM (
    SELECT
        post.id AS post_id,
        coalesce(sum(count_diff), 0) AS comments,
        -- Old rows are excluded using `count_diff = 1`
        max((comment).published_at) FILTER (WHERE count_diff = 1) AS newest_comment_time_at,
        max((comment).published_at) FILTER (WHERE count_diff = 1
            -- Ignore comments from the post's creator
            AND post.creator_id != (comment).creator_id
        -- Ignore comments on old posts
        AND post.published_at > ((comment).published_at - '2 days'::interval)) AS newest_comment_time_necro_at
FROM
     (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    LEFT JOIN post ON post.id = (comment).post_id
WHERE
    r.is_counted (comment)
GROUP BY
    post.id) AS diff
WHERE
    a.id = diff.post_id
    AND (diff.comments,
        GREATEST (a.newest_comment_time_at, diff.newest_comment_time_at),
        GREATEST (a.newest_comment_time_necro_at, diff.newest_comment_time_necro_at)) != (0,
        a.newest_comment_time_at,
        a.newest_comment_time_necro_at);
UPDATE
    local_site AS a
SET
    comments = a.comments + diff.comments
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS comments
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::comment AS comment
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::comment AS comment
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (comment)
        AND (comment).local) AS diff
WHERE
    diff.comments != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.comment_update_statement() OWNER TO lemmy;

--
-- Name: community_actions_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_actions_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community AS a
    SET
        subscribers = a.subscribers + diff.subscribers, subscribers_local = a.subscribers_local + diff.subscribers_local
    FROM (
        SELECT
            (community_actions).community_id, coalesce(sum(count_diff) FILTER (WHERE community.local), 0) AS subscribers, coalesce(sum(count_diff) FILTER (WHERE person.local), 0) AS subscribers_local
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community_actions AS community_actions
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community_actions AS community_actions
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
    LEFT JOIN community ON community.id = (community_actions).community_id
    LEFT JOIN person ON person.id = (community_actions).person_id
    WHERE (community_actions).followed_at IS NOT NULL GROUP BY (community_actions).community_id) AS diff
WHERE
    a.id = diff.community_id
        AND (diff.subscribers, diff.subscribers_local) != (0, 0);
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_actions_delete_statement() OWNER TO lemmy;

--
-- Name: community_actions_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_actions_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community AS a
    SET
        subscribers = a.subscribers + diff.subscribers, subscribers_local = a.subscribers_local + diff.subscribers_local
    FROM (
        SELECT
            (community_actions).community_id, coalesce(sum(count_diff) FILTER (WHERE community.local), 0) AS subscribers, coalesce(sum(count_diff) FILTER (WHERE person.local), 0) AS subscribers_local
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community_actions AS community_actions
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community_actions AS community_actions
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    LEFT JOIN community ON community.id = (community_actions).community_id
    LEFT JOIN person ON person.id = (community_actions).person_id
    WHERE (community_actions).followed_at IS NOT NULL GROUP BY (community_actions).community_id) AS diff
WHERE
    a.id = diff.community_id
        AND (diff.subscribers, diff.subscribers_local) != (0, 0);
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_actions_insert_statement() OWNER TO lemmy;

--
-- Name: community_actions_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_actions_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community AS a
    SET
        subscribers = a.subscribers + diff.subscribers, subscribers_local = a.subscribers_local + diff.subscribers_local
    FROM (
        SELECT
            (community_actions).community_id, coalesce(sum(count_diff) FILTER (WHERE community.local), 0) AS subscribers, coalesce(sum(count_diff) FILTER (WHERE person.local), 0) AS subscribers_local
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community_actions AS community_actions
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community_actions AS community_actions
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    LEFT JOIN community ON community.id = (community_actions).community_id
    LEFT JOIN person ON person.id = (community_actions).person_id
    WHERE (community_actions).followed_at IS NOT NULL GROUP BY (community_actions).community_id) AS diff
WHERE
    a.id = diff.community_id
        AND (diff.subscribers, diff.subscribers_local) != (0, 0);
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_actions_update_statement() OWNER TO lemmy;

--
-- Name: community_aggregates_activity(text); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_aggregates_activity(i text) RETURNS TABLE(count_ bigint, community_id_ integer)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN query
    SELECT
        count(*),
        community_id
    FROM (
        SELECT
            c.creator_id,
            p.community_id
        FROM
            comment c
            INNER JOIN post p ON c.post_id = p.id
            INNER JOIN person pe ON c.creator_id = pe.id
        WHERE
            c.published_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE
        UNION
        SELECT
            p.creator_id,
            p.community_id
        FROM
            post p
            INNER JOIN person pe ON p.creator_id = pe.id
        WHERE
            p.published_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE
        UNION
        SELECT
            pa.person_id,
            p.community_id
        FROM
            post_actions pa
            INNER JOIN post p ON pa.post_id = p.id
            INNER JOIN person pe ON pa.person_id = pe.id
        WHERE
            pa.liked_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE
        UNION
        SELECT
            ca.person_id,
            p.community_id
        FROM
            comment_actions ca
            INNER JOIN comment c ON ca.comment_id = c.id
            INNER JOIN post p ON c.post_id = p.id
            INNER JOIN person pe ON ca.person_id = pe.id
        WHERE
            ca.liked_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE) a
GROUP BY
    community_id;
END;
$$;


ALTER FUNCTION r.community_aggregates_activity(i text) OWNER TO lemmy;

--
-- Name: community_aggregates_interactions(text); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_aggregates_interactions(i text) RETURNS TABLE(count_ bigint, community_id_ integer)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN query
    SELECT
        COALESCE(sum(comments + upvotes + downvotes)::bigint, 0) AS count_,
        community_id AS community_id_
    FROM
        post
    WHERE
        published_at >= (CURRENT_TIMESTAMP - i::interval)
    GROUP BY
        community_id;
END;
$$;


ALTER FUNCTION r.community_aggregates_interactions(i text) OWNER TO lemmy;

--
-- Name: community_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        local_site AS a
    SET
        communities = a.communities + diff.communities
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS communities
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community AS community
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community AS community
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
        WHERE
            r.is_counted (community)
            AND (community).local) AS diff
WHERE
    diff.communities != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_delete_statement() OWNER TO lemmy;

--
-- Name: community_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        local_site AS a
    SET
        communities = a.communities + diff.communities
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS communities
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community AS community
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community AS community
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
        WHERE
            r.is_counted (community)
            AND (community).local) AS diff
WHERE
    diff.communities != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_insert_statement() OWNER TO lemmy;

--
-- Name: community_report_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_report_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (community_report).community_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (community_report).resolved), 0) AS unresolved_report_count
    FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community_report AS community_report
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community_report AS community_report
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows GROUP BY (community_report).community_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
    AND a.id = diff.community_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_report_delete_statement() OWNER TO lemmy;

--
-- Name: community_report_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_report_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (community_report).community_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (community_report).resolved), 0) AS unresolved_report_count
    FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community_report AS community_report
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community_report AS community_report
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows GROUP BY (community_report).community_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
    AND a.id = diff.community_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_report_insert_statement() OWNER TO lemmy;

--
-- Name: community_report_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_report_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (community_report).community_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (community_report).resolved), 0) AS unresolved_report_count
    FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community_report AS community_report
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community_report AS community_report
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows GROUP BY (community_report).community_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
    AND a.id = diff.community_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_report_update_statement() OWNER TO lemmy;

--
-- Name: community_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.community_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        local_site AS a
    SET
        communities = a.communities + diff.communities
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS communities
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::community AS community
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::community AS community
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
        WHERE
            r.is_counted (community)
            AND (community).local) AS diff
WHERE
    diff.communities != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.community_update_statement() OWNER TO lemmy;

--
-- Name: controversy_rank(numeric, numeric); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.controversy_rank(upvotes numeric, downvotes numeric) RETURNS double precision
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    RETURN CASE WHEN ((downvotes <= (0)::numeric) OR (upvotes <= (0)::numeric)) THEN (0)::double precision ELSE (((upvotes + downvotes))::double precision ^ CASE WHEN (upvotes > downvotes) THEN ((downvotes)::double precision / (upvotes)::double precision) ELSE ((upvotes)::double precision / (downvotes)::double precision) END) END;


ALTER FUNCTION r.controversy_rank(upvotes numeric, downvotes numeric) OWNER TO lemmy;

--
-- Name: create_inbox_combined_trigger(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_inbox_combined_trigger(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$ CREATE FUNCTION r.inbox_combined_thing_insert ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                INSERT INTO inbox_combined (published_at, thing_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;
    CREATE TRIGGER inbox_combined
        AFTER INSERT ON thing
        FOR EACH ROW
        EXECUTE FUNCTION r.inbox_combined_thing_insert ( );
        $b$,
        'thing',
        table_name);
END;
$_$;


ALTER PROCEDURE r.create_inbox_combined_trigger(IN table_name text) OWNER TO lemmy;

--
-- Name: create_modlog_combined_trigger(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_modlog_combined_trigger(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$ CREATE FUNCTION r.modlog_combined_thing_insert ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, thing_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;
    CREATE TRIGGER modlog_combined
        AFTER INSERT ON thing
        FOR EACH ROW
        EXECUTE FUNCTION r.modlog_combined_thing_insert ( );
        $b$,
        'thing',
        table_name);
END;
$_$;


ALTER PROCEDURE r.create_modlog_combined_trigger(IN table_name text) OWNER TO lemmy;

--
-- Name: create_person_content_combined_trigger(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_person_content_combined_trigger(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$ CREATE FUNCTION r.person_content_combined_thing_insert ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                INSERT INTO person_content_combined (published_at, thing_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;
    CREATE TRIGGER person_content_combined
        AFTER INSERT ON thing
        FOR EACH ROW
        EXECUTE FUNCTION r.person_content_combined_thing_insert ( );
        $b$,
        'thing',
        table_name);
END;
$_$;


ALTER PROCEDURE r.create_person_content_combined_trigger(IN table_name text) OWNER TO lemmy;

--
-- Name: create_person_liked_combined_trigger(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_person_liked_combined_trigger(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$ CREATE FUNCTION r.person_liked_combined_change_values_thing ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    DELETE FROM person_liked_combined AS p
                    WHERE p.person_id = OLD.person_id
                        AND p.thing_id = OLD.thing_id;
                ELSIF (TG_OP = 'INSERT') THEN
                    IF NEW.liked_at IS NOT NULL AND (
                        SELECT
                            local
                        FROM
                            person
                        WHERE
                            id = NEW.person_id) = TRUE THEN
                        INSERT INTO person_liked_combined (liked_at, like_score, person_id, thing_id)
                            VALUES (NEW.liked_at, NEW.like_score, NEW.person_id, NEW.thing_id);
                    END IF;
                ELSIF (TG_OP = 'UPDATE') THEN
                    IF NEW.liked_at IS NOT NULL AND (
                        SELECT
                            local
                        FROM
                            person
                        WHERE
                            id = NEW.person_id) = TRUE THEN
                        INSERT INTO person_liked_combined (liked_at, like_score, person_id, thing_id)
                            VALUES (NEW.liked_at, NEW.like_score, NEW.person_id, NEW.thing_id);
                        -- If liked gets set as null, delete the row
                    ELSE
                        DELETE FROM person_liked_combined AS p
                        WHERE p.person_id = NEW.person_id
                            AND p.thing_id = NEW.thing_id;
                    END IF;
                END IF;
                RETURN NULL;
            END $$;
    CREATE TRIGGER person_liked_combined
        AFTER INSERT OR DELETE OR UPDATE OF liked_at ON thing_actions
        FOR EACH ROW
        EXECUTE FUNCTION r.person_liked_combined_change_values_thing ( );
    $b$,
    'thing',
    table_name);
END;
$_$;


ALTER PROCEDURE r.create_person_liked_combined_trigger(IN table_name text) OWNER TO lemmy;

--
-- Name: create_person_saved_combined_trigger(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_person_saved_combined_trigger(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$ CREATE FUNCTION r.person_saved_combined_change_values_thing ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    DELETE FROM person_saved_combined AS p
                    WHERE p.person_id = OLD.person_id
                        AND p.thing_id = OLD.thing_id;
                ELSIF (TG_OP = 'INSERT') THEN
                    IF NEW.saved_at IS NOT NULL THEN
                        INSERT INTO person_saved_combined (saved_at, person_id, thing_id)
                            VALUES (NEW.saved_at, NEW.person_id, NEW.thing_id);
                    END IF;
                ELSIF (TG_OP = 'UPDATE') THEN
                    IF NEW.saved_at IS NOT NULL THEN
                        INSERT INTO person_saved_combined (saved_at, person_id, thing_id)
                            VALUES (NEW.saved_at, NEW.person_id, NEW.thing_id);
                        -- If saved gets set as null, delete the row
                    ELSE
                        DELETE FROM person_saved_combined AS p
                        WHERE p.person_id = NEW.person_id
                            AND p.thing_id = NEW.thing_id;
                    END IF;
                END IF;
                RETURN NULL;
            END $$;
    CREATE TRIGGER person_saved_combined
        AFTER INSERT OR DELETE OR UPDATE OF saved_at ON thing_actions
        FOR EACH ROW
        EXECUTE FUNCTION r.person_saved_combined_change_values_thing ( );
    $b$,
    'thing',
    table_name);
END;
$_$;


ALTER PROCEDURE r.create_person_saved_combined_trigger(IN table_name text) OWNER TO lemmy;

--
-- Name: create_report_combined_trigger(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_report_combined_trigger(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$ CREATE FUNCTION r.report_combined_thing_insert ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                INSERT INTO report_combined (published_at, thing_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;
    CREATE TRIGGER report_combined
        AFTER INSERT ON thing
        FOR EACH ROW
        EXECUTE FUNCTION r.report_combined_thing_insert ( );
        $b$,
        'thing',
        table_name);
END;
$_$;


ALTER PROCEDURE r.create_report_combined_trigger(IN table_name text) OWNER TO lemmy;

--
-- Name: create_search_combined_trigger(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_search_combined_trigger(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$ CREATE FUNCTION r.search_combined_thing_insert ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                -- TODO need to figure out how to do the other columns here
                INSERT INTO search_combined (published_at, thing_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;
    CREATE TRIGGER search_combined
        AFTER INSERT ON thing
        FOR EACH ROW
        EXECUTE FUNCTION r.search_combined_thing_insert ( );
        $b$,
        'thing',
        table_name);
END;
$_$;


ALTER PROCEDURE r.create_search_combined_trigger(IN table_name text) OWNER TO lemmy;

--
-- Name: create_triggers(text, text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.create_triggers(IN table_name text, IN function_body text)
    LANGUAGE plpgsql
    AS $_$
DECLARE
    defs text := $$
    -- Delete
    CREATE FUNCTION r.thing_delete_statement ()
        RETURNS TRIGGER
        LANGUAGE plpgsql
        AS function_body_delete;
    CREATE TRIGGER delete_statement
        AFTER DELETE ON thing REFERENCING OLD TABLE AS select_old_rows
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.thing_delete_statement ( );
    -- Insert
    CREATE FUNCTION r.thing_insert_statement ( )
        RETURNS TRIGGER
        LANGUAGE plpgsql
        AS function_body_insert;
    CREATE TRIGGER insert_statement
        AFTER INSERT ON thing REFERENCING NEW TABLE AS select_new_rows
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.thing_insert_statement ( );
    -- Update
    CREATE FUNCTION r.thing_update_statement ( )
        RETURNS TRIGGER
        LANGUAGE plpgsql
        AS function_body_update;
    CREATE TRIGGER update_statement
        AFTER UPDATE ON thing REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.thing_update_statement ( );
    $$;
    select_old_and_new_rows text := $$ (
        SELECT
            -1 AS count_diff,
            old_table::thing AS thing
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::thing AS thing
        FROM
            select_new_rows AS new_table) $$;
    empty_select_new_rows text := $$ (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE) $$;
    empty_select_old_rows text := $$ (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE) $$;
    BEGIN
        function_body := replace(function_body, 'select_old_and_new_rows', select_old_and_new_rows);
        -- `select_old_rows` and `select_new_rows` are made available as empty tables if they don't already exist
        defs := replace(defs, 'function_body_delete', quote_literal(replace(function_body, 'select_new_rows', empty_select_new_rows)));
        defs := replace(defs, 'function_body_insert', quote_literal(replace(function_body, 'select_old_rows', empty_select_old_rows)));
        defs := replace(defs, 'function_body_update', quote_literal(function_body));
        defs := replace(defs, 'thing', table_name);
        EXECUTE defs;
END;
$_$;


ALTER PROCEDURE r.create_triggers(IN table_name text, IN function_body text) OWNER TO lemmy;

--
-- Name: delete_follow_before_person(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.delete_follow_before_person() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM community_actions AS c
    WHERE c.person_id = OLD.id;
    RETURN OLD;
END;
$$;


ALTER FUNCTION r.delete_follow_before_person() OWNER TO lemmy;

--
-- Name: hot_rank(numeric, timestamp with time zone); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.hot_rank(score numeric, published_at timestamp with time zone) RETURNS double precision
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    RETURN CASE WHEN (((now() - published_at) > '00:00:00'::interval) AND ((now() - published_at) < '7 days'::interval)) THEN (log(GREATEST((2)::numeric, (score + (2)::numeric))) / power(((EXTRACT(epoch FROM (now() - published_at)) / (3600)::numeric) + (2)::numeric), 1.8)) ELSE 0.0 END;


ALTER FUNCTION r.hot_rank(score numeric, published_at timestamp with time zone) OWNER TO lemmy;

--
-- Name: inbox_combined_comment_reply_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.inbox_combined_comment_reply_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO inbox_combined (published_at, comment_reply_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.inbox_combined_comment_reply_insert() OWNER TO lemmy;

--
-- Name: inbox_combined_person_comment_mention_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.inbox_combined_person_comment_mention_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO inbox_combined (published_at, person_comment_mention_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.inbox_combined_person_comment_mention_insert() OWNER TO lemmy;

--
-- Name: inbox_combined_person_post_mention_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.inbox_combined_person_post_mention_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO inbox_combined (published_at, person_post_mention_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.inbox_combined_person_post_mention_insert() OWNER TO lemmy;

--
-- Name: inbox_combined_private_message_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.inbox_combined_private_message_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO inbox_combined (published_at, private_message_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.inbox_combined_private_message_insert() OWNER TO lemmy;

--
-- Name: is_counted(record); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.is_counted(item record) RETURNS boolean
    LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    RETURN COALESCE(NOT (item.deleted
            OR item.removed), FALSE);
END;
$$;


ALTER FUNCTION r.is_counted(item record) OWNER TO lemmy;

--
-- Name: local_url(text); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.local_url(url_path text) RETURNS text
    LANGUAGE sql STABLE PARALLEL SAFE
    RETURN (current_setting('lemmy.protocol_and_hostname'::text) || url_path);


ALTER FUNCTION r.local_url(url_path text) OWNER TO lemmy;

--
-- Name: local_user_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.local_user_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        local_site AS a
    SET
        users = a.users + diff.users
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS users
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::local_user AS local_user
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::local_user AS local_user
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
        WHERE (local_user).accepted_application) AS diff
WHERE
    diff.users != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.local_user_delete_statement() OWNER TO lemmy;

--
-- Name: local_user_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.local_user_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        local_site AS a
    SET
        users = a.users + diff.users
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS users
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::local_user AS local_user
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::local_user AS local_user
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
        WHERE (local_user).accepted_application) AS diff
WHERE
    diff.users != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.local_user_insert_statement() OWNER TO lemmy;

--
-- Name: local_user_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.local_user_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        local_site AS a
    SET
        users = a.users + diff.users
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS users
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::local_user AS local_user
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::local_user AS local_user
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
        WHERE (local_user).accepted_application) AS diff
WHERE
    diff.users != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.local_user_update_statement() OWNER TO lemmy;

--
-- Name: modlog_combined_admin_allow_instance_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_admin_allow_instance_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, admin_allow_instance_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_admin_allow_instance_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_admin_block_instance_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_admin_block_instance_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, admin_block_instance_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_admin_block_instance_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_admin_purge_comment_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_admin_purge_comment_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, admin_purge_comment_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_admin_purge_comment_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_admin_purge_community_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_admin_purge_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, admin_purge_community_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_admin_purge_community_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_admin_purge_person_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_admin_purge_person_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, admin_purge_person_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_admin_purge_person_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_admin_purge_post_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_admin_purge_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, admin_purge_post_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_admin_purge_post_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_add_community_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_add_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_add_community_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_add_community_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_add_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_add_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_add_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_add_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_ban_from_community_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_ban_from_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_ban_from_community_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_ban_from_community_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_ban_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_ban_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_ban_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_ban_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_change_community_visibility_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_change_community_visibility_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_change_community_visibility_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_change_community_visibility_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_feature_post_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_feature_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_feature_post_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_feature_post_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_lock_post_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_lock_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_lock_post_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_lock_post_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_remove_comment_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_remove_comment_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_remove_comment_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_remove_comment_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_remove_community_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_remove_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_remove_community_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_remove_community_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_remove_post_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_remove_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_remove_post_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_remove_post_insert() OWNER TO lemmy;

--
-- Name: modlog_combined_mod_transfer_community_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.modlog_combined_mod_transfer_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO modlog_combined (published_at, mod_transfer_community_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.modlog_combined_mod_transfer_community_insert() OWNER TO lemmy;

--
-- Name: parent_comment_ids(public.ltree); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.parent_comment_ids(path public.ltree) RETURNS SETOF integer
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    BEGIN ATOMIC
 SELECT (comment_id.comment_id)::integer AS comment_id
    FROM string_to_table(public.ltree2text(parent_comment_ids.path), '.'::text) comment_id(comment_id)
  OFFSET 1
  LIMIT (public.nlevel(parent_comment_ids.path) - 2);
END;


ALTER FUNCTION r.parent_comment_ids(path public.ltree) OWNER TO lemmy;

--
-- Name: person_content_combined_comment_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.person_content_combined_comment_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO person_content_combined (published_at, comment_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.person_content_combined_comment_insert() OWNER TO lemmy;

--
-- Name: person_content_combined_post_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.person_content_combined_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO person_content_combined (published_at, post_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.person_content_combined_post_insert() OWNER TO lemmy;

--
-- Name: person_liked_combined_change_values_comment(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.person_liked_combined_change_values_comment() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    DELETE FROM person_liked_combined AS p
                    WHERE p.person_id = OLD.person_id
                        AND p.comment_id = OLD.comment_id;
                ELSIF (TG_OP = 'INSERT') THEN
                    IF NEW.liked_at IS NOT NULL AND (
                        SELECT
                            local
                        FROM
                            person
                        WHERE
                            id = NEW.person_id) = TRUE THEN
                        INSERT INTO person_liked_combined (liked_at, like_score, person_id, comment_id)
                            VALUES (NEW.liked_at, NEW.like_score, NEW.person_id, NEW.comment_id);
                    END IF;
                ELSIF (TG_OP = 'UPDATE') THEN
                    IF NEW.liked_at IS NOT NULL AND (
                        SELECT
                            local
                        FROM
                            person
                        WHERE
                            id = NEW.person_id) = TRUE THEN
                        INSERT INTO person_liked_combined (liked_at, like_score, person_id, comment_id)
                            VALUES (NEW.liked_at, NEW.like_score, NEW.person_id, NEW.comment_id);
                        -- If liked gets set as null, delete the row
                    ELSE
                        DELETE FROM person_liked_combined AS p
                        WHERE p.person_id = NEW.person_id
                            AND p.comment_id = NEW.comment_id;
                    END IF;
                END IF;
                RETURN NULL;
            END $$;


ALTER FUNCTION r.person_liked_combined_change_values_comment() OWNER TO lemmy;

--
-- Name: person_liked_combined_change_values_post(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.person_liked_combined_change_values_post() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    DELETE FROM person_liked_combined AS p
                    WHERE p.person_id = OLD.person_id
                        AND p.post_id = OLD.post_id;
                ELSIF (TG_OP = 'INSERT') THEN
                    IF NEW.liked_at IS NOT NULL AND (
                        SELECT
                            local
                        FROM
                            person
                        WHERE
                            id = NEW.person_id) = TRUE THEN
                        INSERT INTO person_liked_combined (liked_at, like_score, person_id, post_id)
                            VALUES (NEW.liked_at, NEW.like_score, NEW.person_id, NEW.post_id);
                    END IF;
                ELSIF (TG_OP = 'UPDATE') THEN
                    IF NEW.liked_at IS NOT NULL AND (
                        SELECT
                            local
                        FROM
                            person
                        WHERE
                            id = NEW.person_id) = TRUE THEN
                        INSERT INTO person_liked_combined (liked_at, like_score, person_id, post_id)
                            VALUES (NEW.liked_at, NEW.like_score, NEW.person_id, NEW.post_id);
                        -- If liked gets set as null, delete the row
                    ELSE
                        DELETE FROM person_liked_combined AS p
                        WHERE p.person_id = NEW.person_id
                            AND p.post_id = NEW.post_id;
                    END IF;
                END IF;
                RETURN NULL;
            END $$;


ALTER FUNCTION r.person_liked_combined_change_values_post() OWNER TO lemmy;

--
-- Name: person_saved_combined_change_values_comment(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.person_saved_combined_change_values_comment() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    DELETE FROM person_saved_combined AS p
                    WHERE p.person_id = OLD.person_id
                        AND p.comment_id = OLD.comment_id;
                ELSIF (TG_OP = 'INSERT') THEN
                    IF NEW.saved_at IS NOT NULL THEN
                        INSERT INTO person_saved_combined (saved_at, person_id, comment_id)
                            VALUES (NEW.saved_at, NEW.person_id, NEW.comment_id);
                    END IF;
                ELSIF (TG_OP = 'UPDATE') THEN
                    IF NEW.saved_at IS NOT NULL THEN
                        INSERT INTO person_saved_combined (saved_at, person_id, comment_id)
                            VALUES (NEW.saved_at, NEW.person_id, NEW.comment_id);
                        -- If saved gets set as null, delete the row
                    ELSE
                        DELETE FROM person_saved_combined AS p
                        WHERE p.person_id = NEW.person_id
                            AND p.comment_id = NEW.comment_id;
                    END IF;
                END IF;
                RETURN NULL;
            END $$;


ALTER FUNCTION r.person_saved_combined_change_values_comment() OWNER TO lemmy;

--
-- Name: person_saved_combined_change_values_post(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.person_saved_combined_change_values_post() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    DELETE FROM person_saved_combined AS p
                    WHERE p.person_id = OLD.person_id
                        AND p.post_id = OLD.post_id;
                ELSIF (TG_OP = 'INSERT') THEN
                    IF NEW.saved_at IS NOT NULL THEN
                        INSERT INTO person_saved_combined (saved_at, person_id, post_id)
                            VALUES (NEW.saved_at, NEW.person_id, NEW.post_id);
                    END IF;
                ELSIF (TG_OP = 'UPDATE') THEN
                    IF NEW.saved_at IS NOT NULL THEN
                        INSERT INTO person_saved_combined (saved_at, person_id, post_id)
                            VALUES (NEW.saved_at, NEW.person_id, NEW.post_id);
                        -- If saved gets set as null, delete the row
                    ELSE
                        DELETE FROM person_saved_combined AS p
                        WHERE p.person_id = NEW.person_id
                            AND p.post_id = NEW.post_id;
                    END IF;
                END IF;
                RETURN NULL;
            END $$;


ALTER FUNCTION r.person_saved_combined_change_values_post() OWNER TO lemmy;

--
-- Name: post_actions_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_actions_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                WITH post_diff AS ( UPDATE
                        post AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (post_actions).post_id, coalesce(sum(count_diff) FILTER (WHERE (post_actions).like_score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (post_actions).like_score != 1), 0) AS downvotes FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post_actions AS post_actions
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post_actions AS post_actions
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
                WHERE (post_actions).like_score IS NOT NULL GROUP BY (post_actions).post_id) AS diff
            WHERE
                a.id = diff.post_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    a.creator_id AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person AS a
            SET
                post_score = a.post_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM post_diff GROUP BY creator_id) AS diff
                WHERE
                    a.id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$;


ALTER FUNCTION r.post_actions_delete_statement() OWNER TO lemmy;

--
-- Name: post_actions_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_actions_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                WITH post_diff AS ( UPDATE
                        post AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (post_actions).post_id, coalesce(sum(count_diff) FILTER (WHERE (post_actions).like_score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (post_actions).like_score != 1), 0) AS downvotes FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post_actions AS post_actions
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post_actions AS post_actions
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
                WHERE (post_actions).like_score IS NOT NULL GROUP BY (post_actions).post_id) AS diff
            WHERE
                a.id = diff.post_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    a.creator_id AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person AS a
            SET
                post_score = a.post_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM post_diff GROUP BY creator_id) AS diff
                WHERE
                    a.id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$;


ALTER FUNCTION r.post_actions_insert_statement() OWNER TO lemmy;

--
-- Name: post_actions_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_actions_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                WITH post_diff AS ( UPDATE
                        post AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (post_actions).post_id, coalesce(sum(count_diff) FILTER (WHERE (post_actions).like_score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (post_actions).like_score != 1), 0) AS downvotes FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post_actions AS post_actions
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post_actions AS post_actions
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
                WHERE (post_actions).like_score IS NOT NULL GROUP BY (post_actions).post_id) AS diff
            WHERE
                a.id = diff.post_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    a.creator_id AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person AS a
            SET
                post_score = a.post_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM post_diff GROUP BY creator_id) AS diff
                WHERE
                    a.id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$;


ALTER FUNCTION r.post_actions_update_statement() OWNER TO lemmy;

--
-- Name: post_change_values(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_change_values() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Set local ap_id
    IF NEW.local THEN
        NEW.ap_id = coalesce(NEW.ap_id, r.local_url ('/post/' || NEW.id::text));
    END IF;
    -- Set aggregates
    NEW.newest_comment_time_at = NEW.published_at;
    NEW.newest_comment_time_necro_at = NEW.published_at;
    RETURN NEW;
END
$$;


ALTER FUNCTION r.post_change_values() OWNER TO lemmy;

--
-- Name: post_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        person AS a
    SET
        post_count = a.post_count + diff.post_count
    FROM (
        SELECT
            (post).creator_id, coalesce(sum(count_diff), 0) AS post_count
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
        WHERE
            r.is_counted (post)
        GROUP BY (post).creator_id) AS diff
WHERE
    a.id = diff.creator_id
        AND diff.post_count != 0;
UPDATE
    community AS a
SET
    posts = a.posts + diff.posts,
    comments = a.comments + diff.comments
FROM (
    SELECT
        (post).community_id,
        coalesce(sum(count_diff), 0) AS posts,
        coalesce(sum(count_diff * (post).comments), 0) AS comments
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (post)
    GROUP BY
        (post).community_id) AS diff
WHERE
    a.id = diff.community_id
    AND (diff.posts,
        diff.comments) != (0,
        0);
UPDATE
    local_site AS a
SET
    posts = a.posts + diff.posts
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS posts
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (post)
        AND (post).local) AS diff
WHERE
    diff.posts != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.post_delete_statement() OWNER TO lemmy;

--
-- Name: post_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        person AS a
    SET
        post_count = a.post_count + diff.post_count
    FROM (
        SELECT
            (post).creator_id, coalesce(sum(count_diff), 0) AS post_count
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
        WHERE
            r.is_counted (post)
        GROUP BY (post).creator_id) AS diff
WHERE
    a.id = diff.creator_id
        AND diff.post_count != 0;
UPDATE
    community AS a
SET
    posts = a.posts + diff.posts,
    comments = a.comments + diff.comments
FROM (
    SELECT
        (post).community_id,
        coalesce(sum(count_diff), 0) AS posts,
        coalesce(sum(count_diff * (post).comments), 0) AS comments
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (post)
    GROUP BY
        (post).community_id) AS diff
WHERE
    a.id = diff.community_id
    AND (diff.posts,
        diff.comments) != (0,
        0);
UPDATE
    local_site AS a
SET
    posts = a.posts + diff.posts
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS posts
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (post)
        AND (post).local) AS diff
WHERE
    diff.posts != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.post_insert_statement() OWNER TO lemmy;

--
-- Name: post_or_comment(text); Type: PROCEDURE; Schema: r; Owner: lemmy
--

CREATE PROCEDURE r.post_or_comment(IN table_name text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
    EXECUTE replace($b$
        -- When a thing gets a vote, update its aggregates and its creator's aggregates
        CALL r.create_triggers ('thing_actions', $$
            BEGIN
                WITH thing_diff AS ( UPDATE
                        thing AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (thing_actions).thing_id, coalesce(sum(count_diff) FILTER (WHERE (thing_actions).like_score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (thing_actions).like_score != 1), 0) AS downvotes FROM select_old_and_new_rows AS old_and_new_rows
                WHERE (thing_actions).like_score IS NOT NULL GROUP BY (thing_actions).thing_id) AS diff
            WHERE
                a.id = diff.thing_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    a.creator_id AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person AS a
            SET
                thing_score = a.thing_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM thing_diff GROUP BY creator_id) AS diff
                WHERE
                    a.id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$);
    $b$,
    'thing',
    table_name);
END;
$_$;


ALTER PROCEDURE r.post_or_comment(IN table_name text) OWNER TO lemmy;

--
-- Name: post_report_delete_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_report_delete_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (post_report).post_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (post_report).resolved
                AND NOT (post_report).violates_instance_rules), 0) AS unresolved_report_count
FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post_report AS post_report
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post_report AS post_report
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE)  AS new_table)  AS old_and_new_rows GROUP BY (post_report).post_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
AND a.id = diff.post_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.post_report_delete_statement() OWNER TO lemmy;

--
-- Name: post_report_insert_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_report_insert_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (post_report).post_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (post_report).resolved
                AND NOT (post_report).violates_instance_rules), 0) AS unresolved_report_count
FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post_report AS post_report
        FROM
             (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE)  AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post_report AS post_report
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows GROUP BY (post_report).post_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
AND a.id = diff.post_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.post_report_insert_statement() OWNER TO lemmy;

--
-- Name: post_report_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_report_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post AS a
    SET
        report_count = a.report_count + diff.report_count, unresolved_report_count = a.unresolved_report_count + diff.unresolved_report_count
    FROM (
        SELECT
            (post_report).post_id, coalesce(sum(count_diff), 0) AS report_count, coalesce(sum(count_diff) FILTER (WHERE NOT (post_report).resolved
                AND NOT (post_report).violates_instance_rules), 0) AS unresolved_report_count
FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post_report AS post_report
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post_report AS post_report
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows GROUP BY (post_report).post_id) AS diff
WHERE (diff.report_count, diff.unresolved_report_count) != (0, 0)
AND a.id = diff.post_id;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.post_report_update_statement() OWNER TO lemmy;

--
-- Name: post_update_statement(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.post_update_statement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        person AS a
    SET
        post_count = a.post_count + diff.post_count
    FROM (
        SELECT
            (post).creator_id, coalesce(sum(count_diff), 0) AS post_count
        FROM  (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
        WHERE
            r.is_counted (post)
        GROUP BY (post).creator_id) AS diff
WHERE
    a.id = diff.creator_id
        AND diff.post_count != 0;
UPDATE
    community AS a
SET
    posts = a.posts + diff.posts,
    comments = a.comments + diff.comments
FROM (
    SELECT
        (post).community_id,
        coalesce(sum(count_diff), 0) AS posts,
        coalesce(sum(count_diff * (post).comments), 0) AS comments
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (post)
    GROUP BY
        (post).community_id) AS diff
WHERE
    a.id = diff.community_id
    AND (diff.posts,
        diff.comments) != (0,
        0);
UPDATE
    local_site AS a
SET
    posts = a.posts + diff.posts
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS posts
    FROM
         (
        SELECT
            -1 AS count_diff,
            old_table::post AS post
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::post AS post
        FROM
            select_new_rows AS new_table)  AS old_and_new_rows
    WHERE
        r.is_counted (post)
        AND (post).local) AS diff
WHERE
    diff.posts != 0;
RETURN NULL;
END;
$$;


ALTER FUNCTION r.post_update_statement() OWNER TO lemmy;

--
-- Name: private_message_change_values(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.private_message_change_values() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Set local ap_id
    IF NEW.local THEN
        NEW.ap_id = coalesce(NEW.ap_id, r.local_url ('/private_message/' || NEW.id::text));
    END IF;
    RETURN NEW;
END
$$;


ALTER FUNCTION r.private_message_change_values() OWNER TO lemmy;

--
-- Name: report_combined_comment_report_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.report_combined_comment_report_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO report_combined (published_at, comment_report_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.report_combined_comment_report_insert() OWNER TO lemmy;

--
-- Name: report_combined_community_report_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.report_combined_community_report_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO report_combined (published_at, community_report_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.report_combined_community_report_insert() OWNER TO lemmy;

--
-- Name: report_combined_post_report_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.report_combined_post_report_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO report_combined (published_at, post_report_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.report_combined_post_report_insert() OWNER TO lemmy;

--
-- Name: report_combined_private_message_report_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.report_combined_private_message_report_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                INSERT INTO report_combined (published_at, private_message_report_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.report_combined_private_message_report_insert() OWNER TO lemmy;

--
-- Name: require_uplete(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.require_uplete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF pg_trigger_depth() = 1 AND NOT starts_with (current_query(), '/**/') THEN
        RAISE 'using delete instead of uplete is not allowed for this table';
    END IF;
    RETURN NULL;
END
$$;


ALTER FUNCTION r.require_uplete() OWNER TO lemmy;

--
-- Name: scaled_rank(numeric, timestamp with time zone, numeric); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.scaled_rank(score numeric, published_at timestamp with time zone, interactions_month numeric) RETURNS double precision
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    RETURN (r.hot_rank(score, published_at) / (log(((2)::numeric + interactions_month)))::double precision);


ALTER FUNCTION r.scaled_rank(score numeric, published_at timestamp with time zone, interactions_month numeric) OWNER TO lemmy;

--
-- Name: search_combined_comment_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_comment_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                -- TODO need to figure out how to do the other columns here
                INSERT INTO search_combined (published_at, comment_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.search_combined_comment_insert() OWNER TO lemmy;

--
-- Name: search_combined_comment_score_update(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_comment_score_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        search_combined
    SET
        score = NEW.score
    WHERE
        comment_id = NEW.id;
    RETURN NULL;
END
$$;


ALTER FUNCTION r.search_combined_comment_score_update() OWNER TO lemmy;

--
-- Name: search_combined_community_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                -- TODO need to figure out how to do the other columns here
                INSERT INTO search_combined (published_at, community_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.search_combined_community_insert() OWNER TO lemmy;

--
-- Name: search_combined_community_score_update(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_community_score_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        search_combined
    SET
        score = NEW.users_active_month
    WHERE
        community_id = NEW.id;
    RETURN NULL;
END
$$;


ALTER FUNCTION r.search_combined_community_score_update() OWNER TO lemmy;

--
-- Name: search_combined_multi_community_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_multi_community_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                -- TODO need to figure out how to do the other columns here
                INSERT INTO search_combined (published_at, multi_community_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.search_combined_multi_community_insert() OWNER TO lemmy;

--
-- Name: search_combined_person_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_person_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                -- TODO need to figure out how to do the other columns here
                INSERT INTO search_combined (published_at, person_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.search_combined_person_insert() OWNER TO lemmy;

--
-- Name: search_combined_person_score_update(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_person_score_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        search_combined
    SET
        score = NEW.post_score
    WHERE
        person_id = NEW.id;
    RETURN NULL;
END
$$;


ALTER FUNCTION r.search_combined_person_score_update() OWNER TO lemmy;

--
-- Name: search_combined_post_insert(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                -- TODO need to figure out how to do the other columns here
                INSERT INTO search_combined (published_at, post_id)
                    VALUES (NEW.published_at, NEW.id);
                RETURN NEW;
            END $$;


ALTER FUNCTION r.search_combined_post_insert() OWNER TO lemmy;

--
-- Name: search_combined_post_score_update(); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.search_combined_post_score_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        search_combined
    SET
        score = NEW.score
    WHERE
        post_id = NEW.id;
    RETURN NULL;
END
$$;


ALTER FUNCTION r.search_combined_post_score_update() OWNER TO lemmy;

--
-- Name: site_aggregates_activity(text); Type: FUNCTION; Schema: r; Owner: lemmy
--

CREATE FUNCTION r.site_aggregates_activity(i text) RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    count_ integer;
BEGIN
    SELECT
        count(*) INTO count_
    FROM (
        SELECT
            c.creator_id
        FROM
            comment c
            INNER JOIN person pe ON c.creator_id = pe.id
        WHERE
            c.published_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE
        UNION
        SELECT
            p.creator_id
        FROM
            post p
            INNER JOIN person pe ON p.creator_id = pe.id
        WHERE
            p.published_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE
        UNION
        SELECT
            pa.person_id
        FROM
            post_actions pa
            INNER JOIN person pe ON pa.person_id = pe.id
        WHERE
            pa.liked_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE
        UNION
        SELECT
            ca.person_id
        FROM
            comment_actions ca
            INNER JOIN person pe ON ca.person_id = pe.id
        WHERE
            ca.liked_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE) a;
    RETURN count_;
END;
$$;


ALTER FUNCTION r.site_aggregates_activity(i text) OWNER TO lemmy;

--
-- Name: restore_views(character varying, character varying); Type: FUNCTION; Schema: utils; Owner: lemmy
--

CREATE FUNCTION utils.restore_views(p_view_schema character varying, p_view_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
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

$$;


ALTER FUNCTION utils.restore_views(p_view_schema character varying, p_view_name character varying) OWNER TO lemmy;

--
-- Name: save_and_drop_views(name, name); Type: FUNCTION; Schema: utils; Owner: lemmy
--

CREATE FUNCTION utils.save_and_drop_views(p_view_schema name, p_view_name name) RETURNS void
    LANGUAGE plpgsql
    AS $$
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
$$;


ALTER FUNCTION utils.save_and_drop_views(p_view_schema name, p_view_name name) OWNER TO lemmy;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.__diesel_schema_migrations OWNER TO lemmy;

--
-- Name: admin_allow_instance; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.admin_allow_instance (
    id integer NOT NULL,
    instance_id integer NOT NULL,
    admin_person_id integer NOT NULL,
    allowed boolean NOT NULL,
    reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admin_allow_instance OWNER TO lemmy;

--
-- Name: admin_allow_instance_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.admin_allow_instance_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.admin_allow_instance_id_seq OWNER TO lemmy;

--
-- Name: admin_allow_instance_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.admin_allow_instance_id_seq OWNED BY public.admin_allow_instance.id;


--
-- Name: admin_block_instance; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.admin_block_instance (
    id integer NOT NULL,
    instance_id integer NOT NULL,
    admin_person_id integer NOT NULL,
    blocked boolean NOT NULL,
    reason text,
    expires_at timestamp with time zone,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admin_block_instance OWNER TO lemmy;

--
-- Name: admin_block_instance_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.admin_block_instance_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.admin_block_instance_id_seq OWNER TO lemmy;

--
-- Name: admin_block_instance_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.admin_block_instance_id_seq OWNED BY public.admin_block_instance.id;


--
-- Name: admin_purge_comment; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.admin_purge_comment (
    id integer NOT NULL,
    admin_person_id integer NOT NULL,
    post_id integer NOT NULL,
    reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admin_purge_comment OWNER TO lemmy;

--
-- Name: admin_purge_comment_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.admin_purge_comment_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.admin_purge_comment_id_seq OWNER TO lemmy;

--
-- Name: admin_purge_comment_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.admin_purge_comment_id_seq OWNED BY public.admin_purge_comment.id;


--
-- Name: admin_purge_community; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.admin_purge_community (
    id integer NOT NULL,
    admin_person_id integer NOT NULL,
    reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admin_purge_community OWNER TO lemmy;

--
-- Name: admin_purge_community_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.admin_purge_community_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.admin_purge_community_id_seq OWNER TO lemmy;

--
-- Name: admin_purge_community_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.admin_purge_community_id_seq OWNED BY public.admin_purge_community.id;


--
-- Name: admin_purge_person; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.admin_purge_person (
    id integer NOT NULL,
    admin_person_id integer NOT NULL,
    reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admin_purge_person OWNER TO lemmy;

--
-- Name: admin_purge_person_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.admin_purge_person_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.admin_purge_person_id_seq OWNER TO lemmy;

--
-- Name: admin_purge_person_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.admin_purge_person_id_seq OWNED BY public.admin_purge_person.id;


--
-- Name: admin_purge_post; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.admin_purge_post (
    id integer NOT NULL,
    admin_person_id integer NOT NULL,
    community_id integer NOT NULL,
    reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.admin_purge_post OWNER TO lemmy;

--
-- Name: admin_purge_post_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.admin_purge_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.admin_purge_post_id_seq OWNER TO lemmy;

--
-- Name: admin_purge_post_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.admin_purge_post_id_seq OWNED BY public.admin_purge_post.id;


--
-- Name: captcha_answer; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.captcha_answer (
    uuid uuid DEFAULT gen_random_uuid() NOT NULL,
    answer text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.captcha_answer OWNER TO lemmy;

--
-- Name: changeme_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.changeme_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
    CYCLE;


ALTER SEQUENCE public.changeme_seq OWNER TO lemmy;

--
-- Name: comment; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.comment (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    post_id integer NOT NULL,
    content text NOT NULL,
    removed boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL,
    ap_id character varying(255) NOT NULL,
    local boolean DEFAULT true NOT NULL,
    path public.ltree DEFAULT '0'::public.ltree NOT NULL,
    distinguished boolean DEFAULT false NOT NULL,
    language_id integer DEFAULT 0 NOT NULL,
    score integer DEFAULT 0 NOT NULL,
    upvotes integer DEFAULT 0 NOT NULL,
    downvotes integer DEFAULT 0 NOT NULL,
    child_count integer DEFAULT 0 NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL,
    federation_pending boolean DEFAULT false NOT NULL
);


ALTER TABLE public.comment OWNER TO lemmy;

--
-- Name: comment_actions; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.comment_actions (
    id integer NOT NULL,
    person_id integer NOT NULL,
    comment_id integer NOT NULL,
    like_score smallint,
    liked_at timestamp with time zone,
    saved_at timestamp with time zone,
    CONSTRAINT comment_actions_check_liked CHECK (((liked_at IS NULL) = (like_score IS NULL)))
);


ALTER TABLE public.comment_actions OWNER TO lemmy;

--
-- Name: comment_actions_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

ALTER TABLE public.comment_actions ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.comment_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: comment_aggregates; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.comment_aggregates (
    comment_id integer NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    upvotes bigint DEFAULT 0 NOT NULL,
    downvotes bigint DEFAULT 0 NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL,
    child_count integer DEFAULT 0 NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL
);


ALTER TABLE public.comment_aggregates OWNER TO lemmy;

--
-- Name: comment_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.comment_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.comment_id_seq OWNER TO lemmy;

--
-- Name: comment_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.comment_id_seq OWNED BY public.comment.id;


--
-- Name: comment_like; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.comment_like (
    person_id integer NOT NULL,
    comment_id integer NOT NULL,
    score smallint NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.comment_like OWNER TO lemmy;

--
-- Name: comment_reply; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.comment_reply (
    id integer NOT NULL,
    recipient_id integer NOT NULL,
    comment_id integer NOT NULL,
    read boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.comment_reply OWNER TO lemmy;

--
-- Name: comment_reply_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.comment_reply_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.comment_reply_id_seq OWNER TO lemmy;

--
-- Name: comment_reply_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.comment_reply_id_seq OWNED BY public.comment_reply.id;


--
-- Name: comment_report; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.comment_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    comment_id integer NOT NULL,
    original_comment_text text NOT NULL,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    violates_instance_rules boolean DEFAULT false NOT NULL
);


ALTER TABLE public.comment_report OWNER TO lemmy;

--
-- Name: comment_report_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.comment_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.comment_report_id_seq OWNER TO lemmy;

--
-- Name: comment_report_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.comment_report_id_seq OWNED BY public.comment_report.id;


--
-- Name: community; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.community (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    title character varying(255) NOT NULL,
    sidebar text,
    removed boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL,
    nsfw boolean DEFAULT false NOT NULL,
    ap_id character varying(255) DEFAULT public.generate_unique_changeme() NOT NULL,
    local boolean DEFAULT true NOT NULL,
    private_key text,
    public_key text NOT NULL,
    last_refreshed_at timestamp with time zone DEFAULT now() NOT NULL,
    icon text,
    banner text,
    followers_url character varying(255) DEFAULT public.generate_unique_changeme(),
    inbox_url character varying(255) DEFAULT public.generate_unique_changeme() NOT NULL,
    posting_restricted_to_mods boolean DEFAULT false NOT NULL,
    instance_id integer NOT NULL,
    moderators_url character varying(255),
    featured_url character varying(255),
    visibility public.community_visibility DEFAULT 'Public'::public.community_visibility NOT NULL,
    description character varying(150),
    random_number smallint DEFAULT public.random_smallint() NOT NULL,
    subscribers integer DEFAULT 0 NOT NULL,
    posts integer DEFAULT 0 NOT NULL,
    comments integer DEFAULT 0 NOT NULL,
    users_active_day integer DEFAULT 0 NOT NULL,
    users_active_week integer DEFAULT 0 NOT NULL,
    users_active_month integer DEFAULT 0 NOT NULL,
    users_active_half_year integer DEFAULT 0 NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    subscribers_local integer DEFAULT 0 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL,
    interactions_month integer DEFAULT 0 NOT NULL,
    local_removed boolean DEFAULT false NOT NULL
);


ALTER TABLE public.community OWNER TO lemmy;

--
-- Name: community_actions; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.community_actions (
    id integer NOT NULL,
    person_id integer NOT NULL,
    community_id integer NOT NULL,
    followed_at timestamp with time zone,
    follow_state public.community_follower_state,
    follow_approver_id integer,
    blocked_at timestamp with time zone,
    became_moderator_at timestamp with time zone,
    received_ban_at timestamp with time zone,
    ban_expires_at timestamp with time zone,
    CONSTRAINT community_actions_check_followed CHECK ((((followed_at IS NULL) = (follow_state IS NULL)) AND (NOT ((followed_at IS NULL) AND (follow_approver_id IS NOT NULL))))),
    CONSTRAINT community_actions_check_received_ban CHECK ((NOT ((received_ban_at IS NULL) AND (ban_expires_at IS NOT NULL))))
);


ALTER TABLE public.community_actions OWNER TO lemmy;

--
-- Name: community_actions_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

ALTER TABLE public.community_actions ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.community_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: community_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.community_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.community_id_seq OWNER TO lemmy;

--
-- Name: community_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.community_id_seq OWNED BY public.community.id;


--
-- Name: community_language; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.community_language (
    community_id integer NOT NULL,
    language_id integer NOT NULL
);


ALTER TABLE public.community_language OWNER TO lemmy;

--
-- Name: community_report; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.community_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    community_id integer NOT NULL,
    original_community_name text NOT NULL,
    original_community_title text NOT NULL,
    original_community_description text,
    original_community_sidebar text,
    original_community_icon text,
    original_community_banner text,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.community_report OWNER TO lemmy;

--
-- Name: community_report_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.community_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.community_report_id_seq OWNER TO lemmy;

--
-- Name: community_report_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.community_report_id_seq OWNED BY public.community_report.id;


--
-- Name: custom_emoji; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.custom_emoji (
    id integer NOT NULL,
    shortcode character varying(128) NOT NULL,
    image_url text NOT NULL,
    alt_text text NOT NULL,
    category text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.custom_emoji OWNER TO lemmy;

--
-- Name: custom_emoji_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.custom_emoji_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.custom_emoji_id_seq OWNER TO lemmy;

--
-- Name: custom_emoji_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.custom_emoji_id_seq OWNED BY public.custom_emoji.id;


--
-- Name: custom_emoji_keyword; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.custom_emoji_keyword (
    custom_emoji_id integer NOT NULL,
    keyword character varying(128) NOT NULL
);


ALTER TABLE public.custom_emoji_keyword OWNER TO lemmy;

--
-- Name: email_verification; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.email_verification (
    id integer NOT NULL,
    local_user_id integer NOT NULL,
    email text NOT NULL,
    verification_token text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.email_verification OWNER TO lemmy;

--
-- Name: email_verification_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.email_verification_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.email_verification_id_seq OWNER TO lemmy;

--
-- Name: email_verification_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.email_verification_id_seq OWNED BY public.email_verification.id;


--
-- Name: federation_allowlist; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.federation_allowlist (
    instance_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.federation_allowlist OWNER TO lemmy;

--
-- Name: federation_blocklist; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.federation_blocklist (
    instance_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    expires_at timestamp with time zone
);


ALTER TABLE public.federation_blocklist OWNER TO lemmy;

--
-- Name: federation_queue_state; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.federation_queue_state (
    instance_id integer NOT NULL,
    last_successful_id bigint,
    fail_count integer NOT NULL,
    last_retry_at timestamp with time zone,
    last_successful_published_time_at timestamp with time zone
);


ALTER TABLE public.federation_queue_state OWNER TO lemmy;

--
-- Name: history_status; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.history_status (
    id integer NOT NULL,
    source text NOT NULL,
    dest text NOT NULL,
    last_scanned_id integer,
    last_scanned_timestamp timestamp with time zone
);


ALTER TABLE public.history_status OWNER TO lemmy;

--
-- Name: history_status_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

ALTER TABLE public.history_status ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.history_status_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: image_details; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.image_details (
    link text NOT NULL,
    width integer NOT NULL,
    height integer NOT NULL,
    content_type text NOT NULL,
    blurhash character varying(50)
);


ALTER TABLE public.image_details OWNER TO lemmy;

--
-- Name: inbox_combined; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.inbox_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    comment_reply_id integer,
    person_comment_mention_id integer,
    person_post_mention_id integer,
    private_message_id integer,
    CONSTRAINT inbox_combined_check CHECK ((num_nonnulls(comment_reply_id, person_comment_mention_id, person_post_mention_id, private_message_id) = 1))
);


ALTER TABLE public.inbox_combined OWNER TO lemmy;

--
-- Name: inbox_combined_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.inbox_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.inbox_combined_id_seq OWNER TO lemmy;

--
-- Name: inbox_combined_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.inbox_combined_id_seq OWNED BY public.inbox_combined.id;


--
-- Name: instance; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.instance (
    id integer NOT NULL,
    domain character varying(255) NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    software character varying(255),
    version character varying(255)
);


ALTER TABLE public.instance OWNER TO lemmy;

--
-- Name: instance_actions; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.instance_actions (
    id integer NOT NULL,
    person_id integer NOT NULL,
    instance_id integer NOT NULL,
    blocked_at timestamp with time zone,
    received_ban_at timestamp with time zone,
    ban_expires_at timestamp with time zone
);


ALTER TABLE public.instance_actions OWNER TO lemmy;

--
-- Name: instance_actions_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

ALTER TABLE public.instance_actions ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.instance_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: instance_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.instance_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.instance_id_seq OWNER TO lemmy;

--
-- Name: instance_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.instance_id_seq OWNED BY public.instance.id;


--
-- Name: language; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.language (
    id integer NOT NULL,
    code character varying(3) NOT NULL,
    name text NOT NULL
);


ALTER TABLE public.language OWNER TO lemmy;

--
-- Name: language_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.language_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.language_id_seq OWNER TO lemmy;

--
-- Name: language_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.language_id_seq OWNED BY public.language.id;


--
-- Name: local_image; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.local_image (
    pictrs_alias text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    person_id integer,
    thumbnail_for_post_id integer
);


ALTER TABLE public.local_image OWNER TO lemmy;

--
-- Name: local_site; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.local_site (
    id integer NOT NULL,
    site_id integer NOT NULL,
    site_setup boolean DEFAULT false NOT NULL,
    community_creation_admin_only boolean DEFAULT false NOT NULL,
    require_email_verification boolean DEFAULT false NOT NULL,
    application_question text DEFAULT 'to verify that you are human, please explain why you want to create an account on this site'::text,
    private_instance boolean DEFAULT false NOT NULL,
    default_theme text DEFAULT 'browser'::text NOT NULL,
    default_post_listing_type public.listing_type_enum DEFAULT 'Local'::public.listing_type_enum NOT NULL,
    legal_information text,
    application_email_admins boolean DEFAULT false NOT NULL,
    slur_filter_regex text,
    actor_name_max_length integer DEFAULT 20 NOT NULL,
    federation_enabled boolean DEFAULT true NOT NULL,
    captcha_enabled boolean DEFAULT false NOT NULL,
    captcha_difficulty character varying(255) DEFAULT 'medium'::character varying NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    registration_mode public.registration_mode_enum DEFAULT 'RequireApplication'::public.registration_mode_enum NOT NULL,
    reports_email_admins boolean DEFAULT false NOT NULL,
    federation_signed_fetch boolean DEFAULT true NOT NULL,
    default_post_listing_mode public.post_listing_mode_enum DEFAULT 'List'::public.post_listing_mode_enum NOT NULL,
    default_post_sort_type public.post_sort_type_enum DEFAULT 'Active'::public.post_sort_type_enum NOT NULL,
    default_comment_sort_type public.comment_sort_type_enum DEFAULT 'Hot'::public.comment_sort_type_enum NOT NULL,
    oauth_registration boolean DEFAULT false NOT NULL,
    post_upvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    post_downvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    comment_upvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    comment_downvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    default_post_time_range_seconds integer,
    disallow_nsfw_content boolean DEFAULT false NOT NULL,
    users integer DEFAULT 1 NOT NULL,
    posts integer DEFAULT 0 NOT NULL,
    comments integer DEFAULT 0 NOT NULL,
    communities integer DEFAULT 0 NOT NULL,
    users_active_day integer DEFAULT 0 NOT NULL,
    users_active_week integer DEFAULT 0 NOT NULL,
    users_active_month integer DEFAULT 0 NOT NULL,
    users_active_half_year integer DEFAULT 0 NOT NULL,
    disable_email_notifications boolean DEFAULT false NOT NULL,
    suggested_communities integer,
    multi_comm_follower integer NOT NULL
);


ALTER TABLE public.local_site OWNER TO lemmy;

--
-- Name: local_site_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.local_site_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.local_site_id_seq OWNER TO lemmy;

--
-- Name: local_site_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.local_site_id_seq OWNED BY public.local_site.id;


--
-- Name: local_site_rate_limit; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.local_site_rate_limit (
    local_site_id integer NOT NULL,
    message_max_requests integer DEFAULT 180 NOT NULL,
    message_interval_seconds integer DEFAULT 60 NOT NULL,
    post_max_requests integer DEFAULT 6 NOT NULL,
    post_interval_seconds integer DEFAULT 600 NOT NULL,
    register_max_requests integer DEFAULT 10 NOT NULL,
    register_interval_seconds integer DEFAULT 3600 NOT NULL,
    image_max_requests integer DEFAULT 6 NOT NULL,
    image_interval_seconds integer DEFAULT 3600 NOT NULL,
    comment_max_requests integer DEFAULT 6 NOT NULL,
    comment_interval_seconds integer DEFAULT 600 NOT NULL,
    search_max_requests integer DEFAULT 60 NOT NULL,
    search_interval_seconds integer DEFAULT 600 NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    import_user_settings_max_requests integer DEFAULT 1 NOT NULL,
    import_user_settings_interval_seconds integer DEFAULT 86400 NOT NULL
);


ALTER TABLE public.local_site_rate_limit OWNER TO lemmy;

--
-- Name: local_site_url_blocklist; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.local_site_url_blocklist (
    id integer NOT NULL,
    url text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.local_site_url_blocklist OWNER TO lemmy;

--
-- Name: local_site_url_blocklist_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.local_site_url_blocklist_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.local_site_url_blocklist_id_seq OWNER TO lemmy;

--
-- Name: local_site_url_blocklist_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.local_site_url_blocklist_id_seq OWNED BY public.local_site_url_blocklist.id;


--
-- Name: local_user; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.local_user (
    id integer NOT NULL,
    person_id integer NOT NULL,
    password_encrypted text,
    email text,
    show_nsfw boolean DEFAULT false NOT NULL,
    theme text DEFAULT 'browser'::text NOT NULL,
    default_post_sort_type public.post_sort_type_enum DEFAULT 'Active'::public.post_sort_type_enum NOT NULL,
    default_listing_type public.listing_type_enum DEFAULT 'Local'::public.listing_type_enum NOT NULL,
    interface_language character varying(20) DEFAULT 'browser'::character varying NOT NULL,
    show_avatars boolean DEFAULT true NOT NULL,
    send_notifications_to_email boolean DEFAULT false NOT NULL,
    show_bot_accounts boolean DEFAULT true NOT NULL,
    show_read_posts boolean DEFAULT true NOT NULL,
    email_verified boolean DEFAULT false NOT NULL,
    accepted_application boolean DEFAULT false NOT NULL,
    totp_2fa_secret text,
    open_links_in_new_tab boolean DEFAULT false NOT NULL,
    blur_nsfw boolean DEFAULT true NOT NULL,
    infinite_scroll_enabled boolean DEFAULT false NOT NULL,
    admin boolean DEFAULT false NOT NULL,
    post_listing_mode public.post_listing_mode_enum DEFAULT 'List'::public.post_listing_mode_enum NOT NULL,
    totp_2fa_enabled boolean DEFAULT false NOT NULL,
    enable_keyboard_navigation boolean DEFAULT false NOT NULL,
    enable_animated_images boolean DEFAULT true NOT NULL,
    enable_private_messages boolean DEFAULT true NOT NULL,
    collapse_bot_comments boolean DEFAULT false NOT NULL,
    default_comment_sort_type public.comment_sort_type_enum DEFAULT 'Hot'::public.comment_sort_type_enum NOT NULL,
    auto_mark_fetched_posts_as_read boolean DEFAULT false NOT NULL,
    last_donation_notification_at timestamp with time zone DEFAULT (now() - (random() * '1 year'::interval)) NOT NULL,
    hide_media boolean DEFAULT false NOT NULL,
    default_post_time_range_seconds integer,
    show_score boolean DEFAULT false NOT NULL,
    show_upvotes boolean DEFAULT true NOT NULL,
    show_downvotes public.vote_show_enum DEFAULT 'Show'::public.vote_show_enum NOT NULL,
    show_upvote_percentage boolean DEFAULT false NOT NULL,
    show_person_votes boolean DEFAULT true NOT NULL
);


ALTER TABLE public.local_user OWNER TO lemmy;

--
-- Name: local_user_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.local_user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.local_user_id_seq OWNER TO lemmy;

--
-- Name: local_user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.local_user_id_seq OWNED BY public.local_user.id;


--
-- Name: local_user_keyword_block; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.local_user_keyword_block (
    local_user_id integer NOT NULL,
    keyword character varying(50) NOT NULL
);


ALTER TABLE public.local_user_keyword_block OWNER TO lemmy;

--
-- Name: local_user_language; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.local_user_language (
    local_user_id integer NOT NULL,
    language_id integer NOT NULL
);


ALTER TABLE public.local_user_language OWNER TO lemmy;

--
-- Name: login_token; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.login_token (
    token text NOT NULL,
    user_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    ip text,
    user_agent text
);


ALTER TABLE public.login_token OWNER TO lemmy;

--
-- Name: mod_add; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_add (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    other_person_id integer NOT NULL,
    removed boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.mod_add OWNER TO lemmy;

--
-- Name: mod_add_community; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_add_community (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    other_person_id integer NOT NULL,
    community_id integer NOT NULL,
    removed boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.mod_add_community OWNER TO lemmy;

--
-- Name: mod_add_community_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_add_community_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_add_community_id_seq OWNER TO lemmy;

--
-- Name: mod_add_community_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_add_community_id_seq OWNED BY public.mod_add_community.id;


--
-- Name: mod_add_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_add_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_add_id_seq OWNER TO lemmy;

--
-- Name: mod_add_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_add_id_seq OWNED BY public.mod_add.id;


--
-- Name: mod_ban; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_ban (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    other_person_id integer NOT NULL,
    reason text,
    banned boolean DEFAULT true NOT NULL,
    expires_at timestamp with time zone,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    instance_id integer NOT NULL
);


ALTER TABLE public.mod_ban OWNER TO lemmy;

--
-- Name: mod_ban_from_community; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_ban_from_community (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    other_person_id integer NOT NULL,
    community_id integer NOT NULL,
    reason text,
    banned boolean DEFAULT true NOT NULL,
    expires_at timestamp with time zone,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.mod_ban_from_community OWNER TO lemmy;

--
-- Name: mod_ban_from_community_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_ban_from_community_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_ban_from_community_id_seq OWNER TO lemmy;

--
-- Name: mod_ban_from_community_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_ban_from_community_id_seq OWNED BY public.mod_ban_from_community.id;


--
-- Name: mod_ban_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_ban_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_ban_id_seq OWNER TO lemmy;

--
-- Name: mod_ban_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_ban_id_seq OWNED BY public.mod_ban.id;


--
-- Name: mod_change_community_visibility; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_change_community_visibility (
    id integer NOT NULL,
    community_id integer NOT NULL,
    mod_person_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    visibility public.community_visibility NOT NULL
);


ALTER TABLE public.mod_change_community_visibility OWNER TO lemmy;

--
-- Name: mod_change_community_visibility_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_change_community_visibility_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_change_community_visibility_id_seq OWNER TO lemmy;

--
-- Name: mod_change_community_visibility_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_change_community_visibility_id_seq OWNED BY public.mod_change_community_visibility.id;


--
-- Name: mod_feature_post; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_feature_post (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    post_id integer NOT NULL,
    featured boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    is_featured_community boolean DEFAULT true NOT NULL
);


ALTER TABLE public.mod_feature_post OWNER TO lemmy;

--
-- Name: mod_lock_post; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_lock_post (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    post_id integer NOT NULL,
    locked boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text
);


ALTER TABLE public.mod_lock_post OWNER TO lemmy;

--
-- Name: mod_lock_post_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_lock_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_lock_post_id_seq OWNER TO lemmy;

--
-- Name: mod_lock_post_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_lock_post_id_seq OWNED BY public.mod_lock_post.id;


--
-- Name: mod_remove_comment; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_remove_comment (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    comment_id integer NOT NULL,
    reason text,
    removed boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.mod_remove_comment OWNER TO lemmy;

--
-- Name: mod_remove_comment_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_remove_comment_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_remove_comment_id_seq OWNER TO lemmy;

--
-- Name: mod_remove_comment_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_remove_comment_id_seq OWNED BY public.mod_remove_comment.id;


--
-- Name: mod_remove_community; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_remove_community (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    community_id integer NOT NULL,
    reason text,
    removed boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.mod_remove_community OWNER TO lemmy;

--
-- Name: mod_remove_community_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_remove_community_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_remove_community_id_seq OWNER TO lemmy;

--
-- Name: mod_remove_community_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_remove_community_id_seq OWNED BY public.mod_remove_community.id;


--
-- Name: mod_remove_post; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_remove_post (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    post_id integer NOT NULL,
    reason text,
    removed boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.mod_remove_post OWNER TO lemmy;

--
-- Name: mod_remove_post_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_remove_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_remove_post_id_seq OWNER TO lemmy;

--
-- Name: mod_remove_post_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_remove_post_id_seq OWNED BY public.mod_remove_post.id;


--
-- Name: mod_sticky_post_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_sticky_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_sticky_post_id_seq OWNER TO lemmy;

--
-- Name: mod_sticky_post_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_sticky_post_id_seq OWNED BY public.mod_feature_post.id;


--
-- Name: mod_transfer_community; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.mod_transfer_community (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    other_person_id integer NOT NULL,
    community_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.mod_transfer_community OWNER TO lemmy;

--
-- Name: mod_transfer_community_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.mod_transfer_community_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.mod_transfer_community_id_seq OWNER TO lemmy;

--
-- Name: mod_transfer_community_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.mod_transfer_community_id_seq OWNED BY public.mod_transfer_community.id;


--
-- Name: modlog_combined; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.modlog_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    admin_allow_instance_id integer,
    admin_block_instance_id integer,
    admin_purge_comment_id integer,
    admin_purge_community_id integer,
    admin_purge_person_id integer,
    admin_purge_post_id integer,
    mod_add_id integer,
    mod_add_community_id integer,
    mod_ban_id integer,
    mod_ban_from_community_id integer,
    mod_feature_post_id integer,
    mod_lock_post_id integer,
    mod_remove_comment_id integer,
    mod_remove_community_id integer,
    mod_remove_post_id integer,
    mod_transfer_community_id integer,
    mod_change_community_visibility_id integer,
    CONSTRAINT modlog_combined_check CHECK ((num_nonnulls(admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_community_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_community_id, mod_ban_id, mod_ban_from_community_id, mod_feature_post_id, mod_change_community_visibility_id, mod_lock_post_id, mod_remove_comment_id, mod_remove_community_id, mod_remove_post_id, mod_transfer_community_id) = 1))
);


ALTER TABLE public.modlog_combined OWNER TO lemmy;

--
-- Name: modlog_combined_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.modlog_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.modlog_combined_id_seq OWNER TO lemmy;

--
-- Name: modlog_combined_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.modlog_combined_id_seq OWNED BY public.modlog_combined.id;


--
-- Name: multi_community; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.multi_community (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    instance_id integer NOT NULL,
    name character varying(255) NOT NULL,
    title character varying(255),
    description character varying(255),
    local boolean DEFAULT true NOT NULL,
    deleted boolean DEFAULT false NOT NULL,
    ap_id text DEFAULT public.generate_unique_changeme() NOT NULL,
    public_key text NOT NULL,
    private_key text,
    inbox_url text DEFAULT public.generate_unique_changeme() NOT NULL,
    last_refreshed_at timestamp with time zone DEFAULT now() NOT NULL,
    following_url text DEFAULT public.generate_unique_changeme() NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.multi_community OWNER TO lemmy;

--
-- Name: multi_community_entry; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.multi_community_entry (
    multi_community_id integer NOT NULL,
    community_id integer NOT NULL
);


ALTER TABLE public.multi_community_entry OWNER TO lemmy;

--
-- Name: multi_community_follow; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.multi_community_follow (
    multi_community_id integer NOT NULL,
    person_id integer NOT NULL,
    follow_state public.community_follower_state NOT NULL
);


ALTER TABLE public.multi_community_follow OWNER TO lemmy;

--
-- Name: multi_community_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.multi_community_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.multi_community_id_seq OWNER TO lemmy;

--
-- Name: multi_community_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.multi_community_id_seq OWNED BY public.multi_community.id;


--
-- Name: oauth_account; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.oauth_account (
    local_user_id integer NOT NULL,
    oauth_provider_id integer NOT NULL,
    oauth_user_id text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.oauth_account OWNER TO lemmy;

--
-- Name: oauth_provider; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.oauth_provider (
    id integer NOT NULL,
    display_name text NOT NULL,
    issuer text NOT NULL,
    authorization_endpoint text NOT NULL,
    token_endpoint text NOT NULL,
    userinfo_endpoint text NOT NULL,
    id_claim text NOT NULL,
    client_id text NOT NULL,
    client_secret text NOT NULL,
    scopes text NOT NULL,
    auto_verify_email boolean DEFAULT true NOT NULL,
    account_linking_enabled boolean DEFAULT false NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    use_pkce boolean DEFAULT false NOT NULL
);


ALTER TABLE public.oauth_provider OWNER TO lemmy;

--
-- Name: oauth_provider_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.oauth_provider_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.oauth_provider_id_seq OWNER TO lemmy;

--
-- Name: oauth_provider_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.oauth_provider_id_seq OWNED BY public.oauth_provider.id;


--
-- Name: password_reset_request; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.password_reset_request (
    id integer NOT NULL,
    token text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    local_user_id integer NOT NULL
);


ALTER TABLE public.password_reset_request OWNER TO lemmy;

--
-- Name: password_reset_request_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.password_reset_request_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.password_reset_request_id_seq OWNER TO lemmy;

--
-- Name: password_reset_request_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.password_reset_request_id_seq OWNED BY public.password_reset_request.id;


--
-- Name: person; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(255),
    avatar text,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    ap_id character varying(255) DEFAULT public.generate_unique_changeme() NOT NULL,
    bio text,
    local boolean DEFAULT true NOT NULL,
    private_key text,
    public_key text NOT NULL,
    last_refreshed_at timestamp with time zone DEFAULT now() NOT NULL,
    banner text,
    deleted boolean DEFAULT false NOT NULL,
    inbox_url character varying(255) DEFAULT public.generate_unique_changeme() NOT NULL,
    matrix_user_id text,
    bot_account boolean DEFAULT false NOT NULL,
    instance_id integer NOT NULL,
    post_count integer DEFAULT 0 NOT NULL,
    post_score integer DEFAULT 0 NOT NULL,
    comment_count integer DEFAULT 0 NOT NULL,
    comment_score integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.person OWNER TO lemmy;

--
-- Name: person_actions; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person_actions (
    id integer NOT NULL,
    person_id integer NOT NULL,
    target_id integer NOT NULL,
    followed_at timestamp with time zone,
    follow_pending boolean,
    blocked_at timestamp with time zone,
    noted_at timestamp with time zone,
    note text,
    voted_at timestamp with time zone,
    upvotes integer,
    downvotes integer,
    CONSTRAINT person_actions_check_followed CHECK (((followed_at IS NULL) = (follow_pending IS NULL)))
);


ALTER TABLE public.person_actions OWNER TO lemmy;

--
-- Name: person_actions_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

ALTER TABLE public.person_actions ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.person_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: person_comment_mention; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person_comment_mention (
    id integer NOT NULL,
    recipient_id integer NOT NULL,
    comment_id integer NOT NULL,
    read boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.person_comment_mention OWNER TO lemmy;

--
-- Name: person_content_combined; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person_content_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    post_id integer,
    comment_id integer,
    CONSTRAINT person_content_combined_check CHECK ((num_nonnulls(post_id, comment_id) = 1))
);


ALTER TABLE public.person_content_combined OWNER TO lemmy;

--
-- Name: person_content_combined_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.person_content_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.person_content_combined_id_seq OWNER TO lemmy;

--
-- Name: person_content_combined_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.person_content_combined_id_seq OWNED BY public.person_content_combined.id;


--
-- Name: person_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.person_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.person_id_seq OWNER TO lemmy;

--
-- Name: person_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.person_id_seq OWNED BY public.person.id;


--
-- Name: person_liked_combined; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person_liked_combined (
    id integer NOT NULL,
    liked_at timestamp with time zone NOT NULL,
    like_score smallint NOT NULL,
    person_id integer NOT NULL,
    post_id integer,
    comment_id integer,
    CONSTRAINT person_liked_combined_check CHECK ((num_nonnulls(post_id, comment_id) = 1))
);


ALTER TABLE public.person_liked_combined OWNER TO lemmy;

--
-- Name: person_liked_combined_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.person_liked_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.person_liked_combined_id_seq OWNER TO lemmy;

--
-- Name: person_liked_combined_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.person_liked_combined_id_seq OWNED BY public.person_liked_combined.id;


--
-- Name: person_mention_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.person_mention_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.person_mention_id_seq OWNER TO lemmy;

--
-- Name: person_mention_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.person_mention_id_seq OWNED BY public.person_comment_mention.id;


--
-- Name: person_post_aggregates; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person_post_aggregates (
    person_id integer NOT NULL,
    post_id integer NOT NULL,
    read_comments bigint DEFAULT 0 NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.person_post_aggregates OWNER TO lemmy;

--
-- Name: person_post_mention; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person_post_mention (
    id integer NOT NULL,
    recipient_id integer NOT NULL,
    post_id integer NOT NULL,
    read boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.person_post_mention OWNER TO lemmy;

--
-- Name: person_post_mention_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.person_post_mention_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.person_post_mention_id_seq OWNER TO lemmy;

--
-- Name: person_post_mention_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.person_post_mention_id_seq OWNED BY public.person_post_mention.id;


--
-- Name: person_saved_combined; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.person_saved_combined (
    id integer NOT NULL,
    saved_at timestamp with time zone NOT NULL,
    person_id integer NOT NULL,
    post_id integer,
    comment_id integer,
    CONSTRAINT person_saved_combined_check CHECK ((num_nonnulls(post_id, comment_id) = 1))
);


ALTER TABLE public.person_saved_combined OWNER TO lemmy;

--
-- Name: person_saved_combined_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.person_saved_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.person_saved_combined_id_seq OWNER TO lemmy;

--
-- Name: person_saved_combined_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.person_saved_combined_id_seq OWNED BY public.person_saved_combined.id;


--
-- Name: post; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.post (
    id integer NOT NULL,
    name character varying(200) NOT NULL,
    url character varying(2000),
    body text,
    creator_id integer NOT NULL,
    community_id integer NOT NULL,
    removed boolean DEFAULT false NOT NULL,
    locked boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL,
    nsfw boolean DEFAULT false NOT NULL,
    embed_title text,
    embed_description text,
    thumbnail_url text,
    ap_id character varying(255) NOT NULL,
    local boolean DEFAULT true NOT NULL,
    embed_video_url text,
    language_id integer DEFAULT 0 NOT NULL,
    featured_community boolean DEFAULT false NOT NULL,
    featured_local boolean DEFAULT false NOT NULL,
    url_content_type text,
    alt_text text,
    scheduled_publish_time_at timestamp with time zone,
    comments integer DEFAULT 0 NOT NULL,
    score integer DEFAULT 0 NOT NULL,
    upvotes integer DEFAULT 0 NOT NULL,
    downvotes integer DEFAULT 0 NOT NULL,
    newest_comment_time_necro_at timestamp with time zone DEFAULT now() NOT NULL,
    newest_comment_time_at timestamp with time zone DEFAULT now() NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    hot_rank_active double precision DEFAULT 0.0001 NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL,
    scaled_rank double precision DEFAULT 0.0001 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL,
    federation_pending boolean DEFAULT false NOT NULL
);


ALTER TABLE public.post OWNER TO lemmy;

--
-- Name: post_actions; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.post_actions (
    id integer NOT NULL,
    person_id integer NOT NULL,
    post_id integer NOT NULL,
    read_at timestamp with time zone,
    read_comments_at timestamp with time zone,
    read_comments_amount integer,
    saved_at timestamp with time zone,
    liked_at timestamp with time zone,
    like_score smallint,
    hidden_at timestamp with time zone,
    CONSTRAINT post_actions_check_liked CHECK (((liked_at IS NULL) = (like_score IS NULL))),
    CONSTRAINT post_actions_check_read_comments CHECK (((read_comments_at IS NULL) = (read_comments_amount IS NULL)))
);


ALTER TABLE public.post_actions OWNER TO lemmy;

--
-- Name: post_actions_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

ALTER TABLE public.post_actions ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.post_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: post_aggregates; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.post_aggregates (
    post_id integer NOT NULL,
    comments bigint DEFAULT 0 NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    upvotes bigint DEFAULT 0 NOT NULL,
    downvotes bigint DEFAULT 0 NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL,
    newest_comment_time_necro timestamp with time zone DEFAULT now() NOT NULL,
    newest_comment_time timestamp with time zone DEFAULT now() NOT NULL,
    featured_community boolean DEFAULT false NOT NULL,
    featured_local boolean DEFAULT false NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    hot_rank_active double precision DEFAULT 0.0001 NOT NULL,
    community_id integer NOT NULL,
    creator_id integer NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL,
    instance_id integer NOT NULL,
    scaled_rank double precision DEFAULT 0.0001 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL
);


ALTER TABLE public.post_aggregates OWNER TO lemmy;

--
-- Name: post_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.post_id_seq OWNER TO lemmy;

--
-- Name: post_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.post_id_seq OWNED BY public.post.id;


--
-- Name: post_like; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.post_like (
    post_id integer NOT NULL,
    person_id integer NOT NULL,
    score smallint NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.post_like OWNER TO lemmy;

--
-- Name: post_read; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.post_read (
    post_id integer NOT NULL,
    person_id integer NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.post_read OWNER TO lemmy;

--
-- Name: post_report; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.post_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    post_id integer NOT NULL,
    original_post_name character varying(200) NOT NULL,
    original_post_url text,
    original_post_body text,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    violates_instance_rules boolean DEFAULT false NOT NULL
);


ALTER TABLE public.post_report OWNER TO lemmy;

--
-- Name: post_report_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.post_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.post_report_id_seq OWNER TO lemmy;

--
-- Name: post_report_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.post_report_id_seq OWNED BY public.post_report.id;


--
-- Name: post_tag; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.post_tag (
    post_id integer NOT NULL,
    tag_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.post_tag OWNER TO lemmy;

--
-- Name: previously_run_sql; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.previously_run_sql (
    id boolean NOT NULL,
    content text NOT NULL
);


ALTER TABLE public.previously_run_sql OWNER TO lemmy;

--
-- Name: private_message; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.private_message (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    recipient_id integer NOT NULL,
    content text NOT NULL,
    deleted boolean DEFAULT false NOT NULL,
    read boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    ap_id character varying(255) NOT NULL,
    local boolean DEFAULT true NOT NULL,
    removed boolean DEFAULT false NOT NULL
);


ALTER TABLE public.private_message OWNER TO lemmy;

--
-- Name: private_message_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.private_message_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.private_message_id_seq OWNER TO lemmy;

--
-- Name: private_message_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.private_message_id_seq OWNED BY public.private_message.id;


--
-- Name: private_message_report; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.private_message_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    private_message_id integer NOT NULL,
    original_pm_text text NOT NULL,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.private_message_report OWNER TO lemmy;

--
-- Name: private_message_report_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.private_message_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.private_message_report_id_seq OWNER TO lemmy;

--
-- Name: private_message_report_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.private_message_report_id_seq OWNED BY public.private_message_report.id;


--
-- Name: received_activity; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.received_activity (
    ap_id text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.received_activity OWNER TO lemmy;

--
-- Name: registration_application; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.registration_application (
    id integer NOT NULL,
    local_user_id integer NOT NULL,
    answer text NOT NULL,
    admin_id integer,
    deny_reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.registration_application OWNER TO lemmy;

--
-- Name: registration_application_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.registration_application_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.registration_application_id_seq OWNER TO lemmy;

--
-- Name: registration_application_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.registration_application_id_seq OWNED BY public.registration_application.id;


--
-- Name: remote_image; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.remote_image (
    link text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.remote_image OWNER TO lemmy;

--
-- Name: report_combined; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.report_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    post_report_id integer,
    comment_report_id integer,
    private_message_report_id integer,
    community_report_id integer,
    CONSTRAINT report_combined_check CHECK ((num_nonnulls(post_report_id, comment_report_id, private_message_report_id, community_report_id) = 1))
);


ALTER TABLE public.report_combined OWNER TO lemmy;

--
-- Name: report_combined_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.report_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.report_combined_id_seq OWNER TO lemmy;

--
-- Name: report_combined_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.report_combined_id_seq OWNED BY public.report_combined.id;


--
-- Name: search_combined; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.search_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    post_id integer,
    comment_id integer,
    community_id integer,
    person_id integer,
    multi_community_id integer,
    CONSTRAINT search_combined_check CHECK ((num_nonnulls(post_id, comment_id, community_id, person_id, multi_community_id) = 1))
);


ALTER TABLE public.search_combined OWNER TO lemmy;

--
-- Name: search_combined_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.search_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.search_combined_id_seq OWNER TO lemmy;

--
-- Name: search_combined_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.search_combined_id_seq OWNED BY public.search_combined.id;


--
-- Name: secret; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.secret (
    id integer NOT NULL,
    jwt_secret character varying DEFAULT gen_random_uuid() NOT NULL
);


ALTER TABLE public.secret OWNER TO lemmy;

--
-- Name: secret_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.secret_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.secret_id_seq OWNER TO lemmy;

--
-- Name: secret_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.secret_id_seq OWNED BY public.secret.id;


--
-- Name: sent_activity; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.sent_activity (
    id bigint NOT NULL,
    ap_id text NOT NULL,
    data json NOT NULL,
    sensitive boolean NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    send_inboxes text[] NOT NULL,
    send_community_followers_of integer,
    send_all_instances boolean NOT NULL,
    actor_type public.actor_type_enum NOT NULL,
    actor_apub_id text
);


ALTER TABLE public.sent_activity OWNER TO lemmy;

--
-- Name: sent_activity_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.sent_activity_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.sent_activity_id_seq OWNER TO lemmy;

--
-- Name: sent_activity_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.sent_activity_id_seq OWNED BY public.sent_activity.id;


--
-- Name: site; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.site (
    id integer NOT NULL,
    name character varying(20) NOT NULL,
    sidebar text,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    icon text,
    banner text,
    description character varying(150),
    ap_id character varying(255) DEFAULT public.generate_unique_changeme() NOT NULL,
    last_refreshed_at timestamp with time zone DEFAULT now() NOT NULL,
    inbox_url character varying(255) DEFAULT public.generate_unique_changeme() NOT NULL,
    private_key text,
    public_key text DEFAULT public.generate_unique_changeme() NOT NULL,
    instance_id integer NOT NULL,
    content_warning text
);


ALTER TABLE public.site OWNER TO lemmy;

--
-- Name: site_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.site_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.site_id_seq OWNER TO lemmy;

--
-- Name: site_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.site_id_seq OWNED BY public.site.id;


--
-- Name: site_language; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.site_language (
    site_id integer NOT NULL,
    language_id integer NOT NULL
);


ALTER TABLE public.site_language OWNER TO lemmy;

--
-- Name: tag; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.tag (
    id integer NOT NULL,
    ap_id text NOT NULL,
    display_name text NOT NULL,
    community_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL
);


ALTER TABLE public.tag OWNER TO lemmy;

--
-- Name: tag_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.tag_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tag_id_seq OWNER TO lemmy;

--
-- Name: tag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.tag_id_seq OWNED BY public.tag.id;


--
-- Name: tagline; Type: TABLE; Schema: public; Owner: lemmy
--

CREATE TABLE public.tagline (
    id integer NOT NULL,
    content text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);


ALTER TABLE public.tagline OWNER TO lemmy;

--
-- Name: tagline_id_seq; Type: SEQUENCE; Schema: public; Owner: lemmy
--

CREATE SEQUENCE public.tagline_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tagline_id_seq OWNER TO lemmy;

--
-- Name: tagline_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: lemmy
--

ALTER SEQUENCE public.tagline_id_seq OWNED BY public.tagline.id;


--
-- Name: deps_saved_ddl; Type: TABLE; Schema: utils; Owner: lemmy
--

CREATE TABLE utils.deps_saved_ddl (
    id integer NOT NULL,
    view_schema character varying(255),
    view_name character varying(255),
    ddl_to_run text
);


ALTER TABLE utils.deps_saved_ddl OWNER TO lemmy;

--
-- Name: deps_saved_ddl_id_seq; Type: SEQUENCE; Schema: utils; Owner: lemmy
--

CREATE SEQUENCE utils.deps_saved_ddl_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE utils.deps_saved_ddl_id_seq OWNER TO lemmy;

--
-- Name: deps_saved_ddl_id_seq; Type: SEQUENCE OWNED BY; Schema: utils; Owner: lemmy
--

ALTER SEQUENCE utils.deps_saved_ddl_id_seq OWNED BY utils.deps_saved_ddl.id;


--
-- Name: admin_allow_instance id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_allow_instance ALTER COLUMN id SET DEFAULT nextval('public.admin_allow_instance_id_seq'::regclass);


--
-- Name: admin_block_instance id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_block_instance ALTER COLUMN id SET DEFAULT nextval('public.admin_block_instance_id_seq'::regclass);


--
-- Name: admin_purge_comment id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_comment ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_comment_id_seq'::regclass);


--
-- Name: admin_purge_community id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_community ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_community_id_seq'::regclass);


--
-- Name: admin_purge_person id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_person ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_person_id_seq'::regclass);


--
-- Name: admin_purge_post id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_post ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_post_id_seq'::regclass);


--
-- Name: comment id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment ALTER COLUMN id SET DEFAULT nextval('public.comment_id_seq'::regclass);


--
-- Name: comment_reply id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_reply ALTER COLUMN id SET DEFAULT nextval('public.comment_reply_id_seq'::regclass);


--
-- Name: comment_report id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_report ALTER COLUMN id SET DEFAULT nextval('public.comment_report_id_seq'::regclass);


--
-- Name: community id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community ALTER COLUMN id SET DEFAULT nextval('public.community_id_seq'::regclass);


--
-- Name: community_report id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_report ALTER COLUMN id SET DEFAULT nextval('public.community_report_id_seq'::regclass);


--
-- Name: custom_emoji id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.custom_emoji ALTER COLUMN id SET DEFAULT nextval('public.custom_emoji_id_seq'::regclass);


--
-- Name: email_verification id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.email_verification ALTER COLUMN id SET DEFAULT nextval('public.email_verification_id_seq'::regclass);


--
-- Name: inbox_combined id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined ALTER COLUMN id SET DEFAULT nextval('public.inbox_combined_id_seq'::regclass);


--
-- Name: instance id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.instance ALTER COLUMN id SET DEFAULT nextval('public.instance_id_seq'::regclass);


--
-- Name: language id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.language ALTER COLUMN id SET DEFAULT nextval('public.language_id_seq'::regclass);


--
-- Name: local_site id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site ALTER COLUMN id SET DEFAULT nextval('public.local_site_id_seq'::regclass);


--
-- Name: local_site_url_blocklist id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site_url_blocklist ALTER COLUMN id SET DEFAULT nextval('public.local_site_url_blocklist_id_seq'::regclass);


--
-- Name: local_user id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user ALTER COLUMN id SET DEFAULT nextval('public.local_user_id_seq'::regclass);


--
-- Name: mod_add id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add ALTER COLUMN id SET DEFAULT nextval('public.mod_add_id_seq'::regclass);


--
-- Name: mod_add_community id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add_community ALTER COLUMN id SET DEFAULT nextval('public.mod_add_community_id_seq'::regclass);


--
-- Name: mod_ban id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban ALTER COLUMN id SET DEFAULT nextval('public.mod_ban_id_seq'::regclass);


--
-- Name: mod_ban_from_community id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban_from_community ALTER COLUMN id SET DEFAULT nextval('public.mod_ban_from_community_id_seq'::regclass);


--
-- Name: mod_change_community_visibility id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_change_community_visibility ALTER COLUMN id SET DEFAULT nextval('public.mod_change_community_visibility_id_seq'::regclass);


--
-- Name: mod_feature_post id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_feature_post ALTER COLUMN id SET DEFAULT nextval('public.mod_sticky_post_id_seq'::regclass);


--
-- Name: mod_lock_post id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_lock_post ALTER COLUMN id SET DEFAULT nextval('public.mod_lock_post_id_seq'::regclass);


--
-- Name: mod_remove_comment id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_comment ALTER COLUMN id SET DEFAULT nextval('public.mod_remove_comment_id_seq'::regclass);


--
-- Name: mod_remove_community id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_community ALTER COLUMN id SET DEFAULT nextval('public.mod_remove_community_id_seq'::regclass);


--
-- Name: mod_remove_post id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_post ALTER COLUMN id SET DEFAULT nextval('public.mod_remove_post_id_seq'::regclass);


--
-- Name: mod_transfer_community id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_transfer_community ALTER COLUMN id SET DEFAULT nextval('public.mod_transfer_community_id_seq'::regclass);


--
-- Name: modlog_combined id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined ALTER COLUMN id SET DEFAULT nextval('public.modlog_combined_id_seq'::regclass);


--
-- Name: multi_community id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community ALTER COLUMN id SET DEFAULT nextval('public.multi_community_id_seq'::regclass);


--
-- Name: oauth_provider id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.oauth_provider ALTER COLUMN id SET DEFAULT nextval('public.oauth_provider_id_seq'::regclass);


--
-- Name: password_reset_request id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.password_reset_request ALTER COLUMN id SET DEFAULT nextval('public.password_reset_request_id_seq'::regclass);


--
-- Name: person id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person ALTER COLUMN id SET DEFAULT nextval('public.person_id_seq'::regclass);


--
-- Name: person_comment_mention id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_comment_mention ALTER COLUMN id SET DEFAULT nextval('public.person_mention_id_seq'::regclass);


--
-- Name: person_content_combined id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_content_combined ALTER COLUMN id SET DEFAULT nextval('public.person_content_combined_id_seq'::regclass);


--
-- Name: person_liked_combined id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_liked_combined ALTER COLUMN id SET DEFAULT nextval('public.person_liked_combined_id_seq'::regclass);


--
-- Name: person_post_mention id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_mention ALTER COLUMN id SET DEFAULT nextval('public.person_post_mention_id_seq'::regclass);


--
-- Name: person_saved_combined id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_saved_combined ALTER COLUMN id SET DEFAULT nextval('public.person_saved_combined_id_seq'::regclass);


--
-- Name: post id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post ALTER COLUMN id SET DEFAULT nextval('public.post_id_seq'::regclass);


--
-- Name: post_report id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_report ALTER COLUMN id SET DEFAULT nextval('public.post_report_id_seq'::regclass);


--
-- Name: private_message id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message ALTER COLUMN id SET DEFAULT nextval('public.private_message_id_seq'::regclass);


--
-- Name: private_message_report id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message_report ALTER COLUMN id SET DEFAULT nextval('public.private_message_report_id_seq'::regclass);


--
-- Name: registration_application id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.registration_application ALTER COLUMN id SET DEFAULT nextval('public.registration_application_id_seq'::regclass);


--
-- Name: report_combined id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined ALTER COLUMN id SET DEFAULT nextval('public.report_combined_id_seq'::regclass);


--
-- Name: search_combined id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined ALTER COLUMN id SET DEFAULT nextval('public.search_combined_id_seq'::regclass);


--
-- Name: secret id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.secret ALTER COLUMN id SET DEFAULT nextval('public.secret_id_seq'::regclass);


--
-- Name: sent_activity id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.sent_activity ALTER COLUMN id SET DEFAULT nextval('public.sent_activity_id_seq'::regclass);


--
-- Name: site id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site ALTER COLUMN id SET DEFAULT nextval('public.site_id_seq'::regclass);


--
-- Name: tag id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.tag ALTER COLUMN id SET DEFAULT nextval('public.tag_id_seq'::regclass);


--
-- Name: tagline id; Type: DEFAULT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.tagline ALTER COLUMN id SET DEFAULT nextval('public.tagline_id_seq'::regclass);


--
-- Name: deps_saved_ddl id; Type: DEFAULT; Schema: utils; Owner: lemmy
--

ALTER TABLE ONLY utils.deps_saved_ddl ALTER COLUMN id SET DEFAULT nextval('utils.deps_saved_ddl_id_seq'::regclass);


--
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: admin_allow_instance admin_allow_instance_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_allow_instance
    ADD CONSTRAINT admin_allow_instance_pkey PRIMARY KEY (id);


--
-- Name: admin_block_instance admin_block_instance_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_block_instance
    ADD CONSTRAINT admin_block_instance_pkey PRIMARY KEY (id);


--
-- Name: admin_purge_comment admin_purge_comment_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_comment
    ADD CONSTRAINT admin_purge_comment_pkey PRIMARY KEY (id);


--
-- Name: admin_purge_community admin_purge_community_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_community
    ADD CONSTRAINT admin_purge_community_pkey PRIMARY KEY (id);


--
-- Name: admin_purge_person admin_purge_person_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_person
    ADD CONSTRAINT admin_purge_person_pkey PRIMARY KEY (id);


--
-- Name: admin_purge_post admin_purge_post_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_post
    ADD CONSTRAINT admin_purge_post_pkey PRIMARY KEY (id);


--
-- Name: captcha_answer captcha_answer_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.captcha_answer
    ADD CONSTRAINT captcha_answer_pkey PRIMARY KEY (uuid);


--
-- Name: comment_actions comment_actions_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_actions
    ADD CONSTRAINT comment_actions_id_key UNIQUE (id);


--
-- Name: comment_actions comment_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_actions
    ADD CONSTRAINT comment_actions_pkey PRIMARY KEY (person_id, comment_id);


--
-- Name: comment_aggregates comment_aggregates_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_aggregates
    ADD CONSTRAINT comment_aggregates_pkey PRIMARY KEY (comment_id);


--
-- Name: comment_like comment_like_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_like
    ADD CONSTRAINT comment_like_pkey PRIMARY KEY (person_id, comment_id);


--
-- Name: comment comment_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment
    ADD CONSTRAINT comment_pkey PRIMARY KEY (id);


--
-- Name: comment_reply comment_reply_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_reply
    ADD CONSTRAINT comment_reply_pkey PRIMARY KEY (id);


--
-- Name: comment_reply comment_reply_recipient_id_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_reply
    ADD CONSTRAINT comment_reply_recipient_id_comment_id_key UNIQUE (recipient_id, comment_id);


--
-- Name: comment_report comment_report_comment_id_creator_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_report
    ADD CONSTRAINT comment_report_comment_id_creator_id_key UNIQUE (comment_id, creator_id);


--
-- Name: comment_report comment_report_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_report
    ADD CONSTRAINT comment_report_pkey PRIMARY KEY (id);


--
-- Name: community_actions community_actions_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_actions
    ADD CONSTRAINT community_actions_id_key UNIQUE (id);


--
-- Name: community_actions community_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_actions
    ADD CONSTRAINT community_actions_pkey PRIMARY KEY (person_id, community_id);


--
-- Name: community community_featured_url_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community
    ADD CONSTRAINT community_featured_url_key UNIQUE (featured_url);


--
-- Name: community_language community_language_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_language
    ADD CONSTRAINT community_language_pkey PRIMARY KEY (community_id, language_id);


--
-- Name: community community_moderators_url_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community
    ADD CONSTRAINT community_moderators_url_key UNIQUE (moderators_url);


--
-- Name: community community_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community
    ADD CONSTRAINT community_pkey PRIMARY KEY (id);


--
-- Name: community_report community_report_community_id_creator_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_report
    ADD CONSTRAINT community_report_community_id_creator_id_key UNIQUE (community_id, creator_id);


--
-- Name: community_report community_report_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_report
    ADD CONSTRAINT community_report_pkey PRIMARY KEY (id);


--
-- Name: custom_emoji custom_emoji_image_url_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.custom_emoji
    ADD CONSTRAINT custom_emoji_image_url_key UNIQUE (image_url);


--
-- Name: custom_emoji_keyword custom_emoji_keyword_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.custom_emoji_keyword
    ADD CONSTRAINT custom_emoji_keyword_pkey PRIMARY KEY (custom_emoji_id, keyword);


--
-- Name: custom_emoji custom_emoji_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.custom_emoji
    ADD CONSTRAINT custom_emoji_pkey PRIMARY KEY (id);


--
-- Name: custom_emoji custom_emoji_shortcode_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.custom_emoji
    ADD CONSTRAINT custom_emoji_shortcode_key UNIQUE (shortcode);


--
-- Name: email_verification email_verification_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.email_verification
    ADD CONSTRAINT email_verification_pkey PRIMARY KEY (id);


--
-- Name: federation_allowlist federation_allowlist_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.federation_allowlist
    ADD CONSTRAINT federation_allowlist_pkey PRIMARY KEY (instance_id);


--
-- Name: federation_blocklist federation_blocklist_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.federation_blocklist
    ADD CONSTRAINT federation_blocklist_pkey PRIMARY KEY (instance_id);


--
-- Name: federation_queue_state federation_queue_state_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.federation_queue_state
    ADD CONSTRAINT federation_queue_state_pkey PRIMARY KEY (instance_id);


--
-- Name: history_status history_status_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.history_status
    ADD CONSTRAINT history_status_id_key UNIQUE (id);


--
-- Name: history_status history_status_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.history_status
    ADD CONSTRAINT history_status_pkey PRIMARY KEY (source, dest);


--
-- Name: comment idx_comment_ap_id; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment
    ADD CONSTRAINT idx_comment_ap_id UNIQUE (ap_id);


--
-- Name: community idx_community_actor_id; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community
    ADD CONSTRAINT idx_community_actor_id UNIQUE (ap_id);


--
-- Name: community idx_community_followers_url; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community
    ADD CONSTRAINT idx_community_followers_url UNIQUE (followers_url);


--
-- Name: person idx_person_actor_id; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person
    ADD CONSTRAINT idx_person_actor_id UNIQUE (ap_id);


--
-- Name: post idx_post_ap_id; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post
    ADD CONSTRAINT idx_post_ap_id UNIQUE (ap_id);


--
-- Name: private_message idx_private_message_ap_id; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message
    ADD CONSTRAINT idx_private_message_ap_id UNIQUE (ap_id);


--
-- Name: site idx_site_instance_unique; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site
    ADD CONSTRAINT idx_site_instance_unique UNIQUE (instance_id);


--
-- Name: image_details image_details_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.image_details
    ADD CONSTRAINT image_details_pkey PRIMARY KEY (link);


--
-- Name: local_image image_upload_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_image
    ADD CONSTRAINT image_upload_pkey PRIMARY KEY (pictrs_alias);


--
-- Name: inbox_combined inbox_combined_comment_reply_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_comment_reply_id_key UNIQUE (comment_reply_id);


--
-- Name: inbox_combined inbox_combined_person_comment_mention_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_comment_mention_id_key UNIQUE (person_comment_mention_id);


--
-- Name: inbox_combined inbox_combined_person_post_mention_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_post_mention_id_key UNIQUE (person_post_mention_id);


--
-- Name: inbox_combined inbox_combined_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_pkey PRIMARY KEY (id);


--
-- Name: inbox_combined inbox_combined_private_message_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_private_message_id_key UNIQUE (private_message_id);


--
-- Name: instance_actions instance_actions_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.instance_actions
    ADD CONSTRAINT instance_actions_id_key UNIQUE (id);


--
-- Name: instance_actions instance_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.instance_actions
    ADD CONSTRAINT instance_actions_pkey PRIMARY KEY (person_id, instance_id);


--
-- Name: instance instance_domain_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.instance
    ADD CONSTRAINT instance_domain_key UNIQUE (domain);


--
-- Name: instance instance_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.instance
    ADD CONSTRAINT instance_pkey PRIMARY KEY (id);


--
-- Name: language language_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.language
    ADD CONSTRAINT language_pkey PRIMARY KEY (id);


--
-- Name: local_site local_site_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_pkey PRIMARY KEY (id);


--
-- Name: local_site_rate_limit local_site_rate_limit_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site_rate_limit
    ADD CONSTRAINT local_site_rate_limit_pkey PRIMARY KEY (local_site_id);


--
-- Name: local_site local_site_site_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_site_id_key UNIQUE (site_id);


--
-- Name: local_site_url_blocklist local_site_url_blocklist_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site_url_blocklist
    ADD CONSTRAINT local_site_url_blocklist_pkey PRIMARY KEY (id);


--
-- Name: local_site_url_blocklist local_site_url_blocklist_url_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site_url_blocklist
    ADD CONSTRAINT local_site_url_blocklist_url_key UNIQUE (url);


--
-- Name: local_user local_user_email_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_email_key UNIQUE (email);


--
-- Name: local_user_keyword_block local_user_keyword_block_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user_keyword_block
    ADD CONSTRAINT local_user_keyword_block_pkey PRIMARY KEY (local_user_id, keyword);


--
-- Name: local_user_language local_user_language_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user_language
    ADD CONSTRAINT local_user_language_pkey PRIMARY KEY (local_user_id, language_id);


--
-- Name: local_user local_user_person_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_person_id_key UNIQUE (person_id);


--
-- Name: local_user local_user_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_pkey PRIMARY KEY (id);


--
-- Name: login_token login_token_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.login_token
    ADD CONSTRAINT login_token_pkey PRIMARY KEY (token);


--
-- Name: mod_add_community mod_add_community_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add_community
    ADD CONSTRAINT mod_add_community_pkey PRIMARY KEY (id);


--
-- Name: mod_add mod_add_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add
    ADD CONSTRAINT mod_add_pkey PRIMARY KEY (id);


--
-- Name: mod_ban_from_community mod_ban_from_community_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban_from_community
    ADD CONSTRAINT mod_ban_from_community_pkey PRIMARY KEY (id);


--
-- Name: mod_ban mod_ban_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban
    ADD CONSTRAINT mod_ban_pkey PRIMARY KEY (id);


--
-- Name: mod_change_community_visibility mod_change_community_visibility_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_change_community_visibility
    ADD CONSTRAINT mod_change_community_visibility_pkey PRIMARY KEY (id);


--
-- Name: mod_lock_post mod_lock_post_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_lock_post
    ADD CONSTRAINT mod_lock_post_pkey PRIMARY KEY (id);


--
-- Name: mod_remove_comment mod_remove_comment_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_comment
    ADD CONSTRAINT mod_remove_comment_pkey PRIMARY KEY (id);


--
-- Name: mod_remove_community mod_remove_community_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_community
    ADD CONSTRAINT mod_remove_community_pkey PRIMARY KEY (id);


--
-- Name: mod_remove_post mod_remove_post_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_post
    ADD CONSTRAINT mod_remove_post_pkey PRIMARY KEY (id);


--
-- Name: mod_feature_post mod_sticky_post_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_feature_post
    ADD CONSTRAINT mod_sticky_post_pkey PRIMARY KEY (id);


--
-- Name: mod_transfer_community mod_transfer_community_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_transfer_community
    ADD CONSTRAINT mod_transfer_community_pkey PRIMARY KEY (id);


--
-- Name: modlog_combined modlog_combined_admin_allow_instance_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_allow_instance_id_key UNIQUE (admin_allow_instance_id);


--
-- Name: modlog_combined modlog_combined_admin_block_instance_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_block_instance_id_key UNIQUE (admin_block_instance_id);


--
-- Name: modlog_combined modlog_combined_admin_purge_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_comment_id_key UNIQUE (admin_purge_comment_id);


--
-- Name: modlog_combined modlog_combined_admin_purge_community_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_community_id_key UNIQUE (admin_purge_community_id);


--
-- Name: modlog_combined modlog_combined_admin_purge_person_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_person_id_key UNIQUE (admin_purge_person_id);


--
-- Name: modlog_combined modlog_combined_admin_purge_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_post_id_key UNIQUE (admin_purge_post_id);


--
-- Name: modlog_combined modlog_combined_mod_add_community_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_community_id_key UNIQUE (mod_add_community_id);


--
-- Name: modlog_combined modlog_combined_mod_add_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_id_key UNIQUE (mod_add_id);


--
-- Name: modlog_combined modlog_combined_mod_ban_from_community_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_from_community_id_key UNIQUE (mod_ban_from_community_id);


--
-- Name: modlog_combined modlog_combined_mod_ban_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_id_key UNIQUE (mod_ban_id);


--
-- Name: modlog_combined modlog_combined_mod_feature_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_feature_post_id_key UNIQUE (mod_feature_post_id);


--
-- Name: modlog_combined modlog_combined_mod_lock_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_key UNIQUE (mod_lock_post_id);


--
-- Name: modlog_combined modlog_combined_mod_remove_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_comment_id_key UNIQUE (mod_remove_comment_id);


--
-- Name: modlog_combined modlog_combined_mod_remove_community_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_community_id_key UNIQUE (mod_remove_community_id);


--
-- Name: modlog_combined modlog_combined_mod_remove_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_key UNIQUE (mod_remove_post_id);


--
-- Name: modlog_combined modlog_combined_mod_transfer_community_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_transfer_community_id_key UNIQUE (mod_transfer_community_id);


--
-- Name: modlog_combined modlog_combined_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_pkey PRIMARY KEY (id);


--
-- Name: multi_community multi_community_ap_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community
    ADD CONSTRAINT multi_community_ap_id_key UNIQUE (ap_id);


--
-- Name: multi_community_entry multi_community_entry_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community_entry
    ADD CONSTRAINT multi_community_entry_pkey PRIMARY KEY (multi_community_id, community_id);


--
-- Name: multi_community_follow multi_community_follow_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community_follow
    ADD CONSTRAINT multi_community_follow_pkey PRIMARY KEY (person_id, multi_community_id);


--
-- Name: multi_community multi_community_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community
    ADD CONSTRAINT multi_community_pkey PRIMARY KEY (id);


--
-- Name: oauth_account oauth_account_oauth_provider_id_oauth_user_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_oauth_provider_id_oauth_user_id_key UNIQUE (oauth_provider_id, oauth_user_id);


--
-- Name: oauth_account oauth_account_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_pkey PRIMARY KEY (oauth_provider_id, local_user_id);


--
-- Name: oauth_provider oauth_provider_client_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.oauth_provider
    ADD CONSTRAINT oauth_provider_client_id_key UNIQUE (client_id);


--
-- Name: oauth_provider oauth_provider_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.oauth_provider
    ADD CONSTRAINT oauth_provider_pkey PRIMARY KEY (id);


--
-- Name: password_reset_request password_reset_request_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.password_reset_request
    ADD CONSTRAINT password_reset_request_pkey PRIMARY KEY (id);


--
-- Name: person person__pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person
    ADD CONSTRAINT person__pkey PRIMARY KEY (id);


--
-- Name: person_actions person_actions_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_actions
    ADD CONSTRAINT person_actions_id_key UNIQUE (id);


--
-- Name: person_actions person_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_actions
    ADD CONSTRAINT person_actions_pkey PRIMARY KEY (person_id, target_id);


--
-- Name: person_content_combined person_content_combined_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_comment_id_key UNIQUE (comment_id);


--
-- Name: person_content_combined person_content_combined_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_pkey PRIMARY KEY (id);


--
-- Name: person_content_combined person_content_combined_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_post_id_key UNIQUE (post_id);


--
-- Name: person_liked_combined person_liked_combined_person_id_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_person_id_comment_id_key UNIQUE (person_id, comment_id);


--
-- Name: person_liked_combined person_liked_combined_person_id_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_person_id_post_id_key UNIQUE (person_id, post_id);


--
-- Name: person_liked_combined person_liked_combined_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_pkey PRIMARY KEY (id);


--
-- Name: person_comment_mention person_mention_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_comment_mention
    ADD CONSTRAINT person_mention_pkey PRIMARY KEY (id);


--
-- Name: person_comment_mention person_mention_recipient_id_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_comment_mention
    ADD CONSTRAINT person_mention_recipient_id_comment_id_key UNIQUE (recipient_id, comment_id);


--
-- Name: person_post_aggregates person_post_aggregates_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_aggregates
    ADD CONSTRAINT person_post_aggregates_pkey PRIMARY KEY (person_id, post_id);


--
-- Name: person_post_mention person_post_mention_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_pkey PRIMARY KEY (id);


--
-- Name: person_post_mention person_post_mention_unique; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_unique UNIQUE (recipient_id, post_id);


--
-- Name: person_saved_combined person_saved_combined_person_id_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_person_id_comment_id_key UNIQUE (person_id, comment_id);


--
-- Name: person_saved_combined person_saved_combined_person_id_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_person_id_post_id_key UNIQUE (person_id, post_id);


--
-- Name: person_saved_combined person_saved_combined_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_pkey PRIMARY KEY (id);


--
-- Name: post_actions post_actions_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_actions
    ADD CONSTRAINT post_actions_id_key UNIQUE (id);


--
-- Name: post_actions post_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_actions
    ADD CONSTRAINT post_actions_pkey PRIMARY KEY (person_id, post_id);


--
-- Name: post_aggregates post_aggregates_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_aggregates
    ADD CONSTRAINT post_aggregates_pkey PRIMARY KEY (post_id);


--
-- Name: post_like post_like_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_like
    ADD CONSTRAINT post_like_pkey PRIMARY KEY (person_id, post_id);


--
-- Name: post post_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_pkey PRIMARY KEY (id);


--
-- Name: post_read post_read_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_read
    ADD CONSTRAINT post_read_pkey PRIMARY KEY (person_id, post_id);


--
-- Name: post_report post_report_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_pkey PRIMARY KEY (id);


--
-- Name: post_report post_report_post_id_creator_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_post_id_creator_id_key UNIQUE (post_id, creator_id);


--
-- Name: post_tag post_tag_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_tag
    ADD CONSTRAINT post_tag_pkey PRIMARY KEY (post_id, tag_id);


--
-- Name: previously_run_sql previously_run_sql_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.previously_run_sql
    ADD CONSTRAINT previously_run_sql_pkey PRIMARY KEY (id);


--
-- Name: private_message private_message_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message
    ADD CONSTRAINT private_message_pkey PRIMARY KEY (id);


--
-- Name: private_message_report private_message_report_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message_report
    ADD CONSTRAINT private_message_report_pkey PRIMARY KEY (id);


--
-- Name: private_message_report private_message_report_private_message_id_creator_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message_report
    ADD CONSTRAINT private_message_report_private_message_id_creator_id_key UNIQUE (private_message_id, creator_id);


--
-- Name: received_activity received_activity_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.received_activity
    ADD CONSTRAINT received_activity_pkey PRIMARY KEY (ap_id);


--
-- Name: registration_application registration_application_local_user_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.registration_application
    ADD CONSTRAINT registration_application_local_user_id_key UNIQUE (local_user_id);


--
-- Name: registration_application registration_application_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.registration_application
    ADD CONSTRAINT registration_application_pkey PRIMARY KEY (id);


--
-- Name: remote_image remote_image_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.remote_image
    ADD CONSTRAINT remote_image_pkey PRIMARY KEY (link);


--
-- Name: report_combined report_combined_comment_report_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_comment_report_id_key UNIQUE (comment_report_id);


--
-- Name: report_combined report_combined_community_report_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_community_report_id_key UNIQUE (community_report_id);


--
-- Name: report_combined report_combined_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_pkey PRIMARY KEY (id);


--
-- Name: report_combined report_combined_post_report_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_post_report_id_key UNIQUE (post_report_id);


--
-- Name: report_combined report_combined_private_message_report_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_private_message_report_id_key UNIQUE (private_message_report_id);


--
-- Name: search_combined search_combined_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_comment_id_key UNIQUE (comment_id);


--
-- Name: search_combined search_combined_community_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_community_id_key UNIQUE (community_id);


--
-- Name: search_combined search_combined_person_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_person_id_key UNIQUE (person_id);


--
-- Name: search_combined search_combined_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_pkey PRIMARY KEY (id);


--
-- Name: search_combined search_combined_post_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_post_id_key UNIQUE (post_id);


--
-- Name: secret secret_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.secret
    ADD CONSTRAINT secret_pkey PRIMARY KEY (id);


--
-- Name: sent_activity sent_activity_ap_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.sent_activity
    ADD CONSTRAINT sent_activity_ap_id_key UNIQUE (ap_id);


--
-- Name: sent_activity sent_activity_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.sent_activity
    ADD CONSTRAINT sent_activity_pkey PRIMARY KEY (id);


--
-- Name: site site_actor_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site
    ADD CONSTRAINT site_actor_id_key UNIQUE (ap_id);


--
-- Name: site_language site_language_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site_language
    ADD CONSTRAINT site_language_pkey PRIMARY KEY (site_id, language_id);


--
-- Name: site site_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site
    ADD CONSTRAINT site_pkey PRIMARY KEY (id);


--
-- Name: tag tag_ap_id_key; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_ap_id_key UNIQUE (ap_id);


--
-- Name: tag tag_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_pkey PRIMARY KEY (id);


--
-- Name: tagline tagline_pkey; Type: CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.tagline
    ADD CONSTRAINT tagline_pkey PRIMARY KEY (id);


--
-- Name: deps_saved_ddl deps_saved_ddl_pkey; Type: CONSTRAINT; Schema: utils; Owner: lemmy
--

ALTER TABLE ONLY utils.deps_saved_ddl
    ADD CONSTRAINT deps_saved_ddl_pkey PRIMARY KEY (id);


--
-- Name: idx_comment_actions_comment; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_actions_comment ON public.comment_actions USING btree (comment_id);


--
-- Name: idx_comment_actions_like_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_actions_like_score ON public.comment_actions USING btree (comment_id, like_score, person_id) WHERE (like_score IS NOT NULL);


--
-- Name: idx_comment_actions_liked_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_actions_liked_not_null ON public.comment_actions USING btree (person_id, comment_id) WHERE ((liked_at IS NOT NULL) OR (like_score IS NOT NULL));


--
-- Name: idx_comment_actions_person; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_actions_person ON public.comment_actions USING btree (person_id);


--
-- Name: idx_comment_actions_saved_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_actions_saved_not_null ON public.comment_actions USING btree (person_id, comment_id) WHERE (saved_at IS NOT NULL);


--
-- Name: idx_comment_aggregates_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_aggregates_controversy ON public.comment_aggregates USING btree (controversy_rank DESC);


--
-- Name: idx_comment_aggregates_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_aggregates_hot ON public.comment_aggregates USING btree (hot_rank DESC, score DESC);


--
-- Name: idx_comment_aggregates_nonzero_hotrank; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_aggregates_nonzero_hotrank ON public.comment_aggregates USING btree (published) WHERE (hot_rank <> (0)::double precision);


--
-- Name: idx_comment_aggregates_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_aggregates_published ON public.comment_aggregates USING btree (published DESC);


--
-- Name: idx_comment_aggregates_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_aggregates_score ON public.comment_aggregates USING btree (score DESC);


--
-- Name: idx_comment_content_trigram; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_content_trigram ON public.comment USING gin (content public.gin_trgm_ops);


--
-- Name: idx_comment_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_controversy ON public.comment USING btree (controversy_rank DESC);


--
-- Name: idx_comment_creator; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_creator ON public.comment USING btree (creator_id);


--
-- Name: idx_comment_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_hot ON public.comment USING btree (hot_rank DESC, score DESC);


--
-- Name: idx_comment_language; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_language ON public.comment USING btree (language_id);


--
-- Name: idx_comment_like_comment; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_like_comment ON public.comment_like USING btree (comment_id);


--
-- Name: idx_comment_nonzero_hotrank; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_nonzero_hotrank ON public.comment USING btree (published_at) WHERE (hot_rank <> (0)::double precision);


--
-- Name: idx_comment_post; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_post ON public.comment USING btree (post_id);


--
-- Name: idx_comment_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_published ON public.comment USING btree (published_at DESC);


--
-- Name: idx_comment_reply_comment; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_reply_comment ON public.comment_reply USING btree (comment_id);


--
-- Name: idx_comment_reply_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_reply_published ON public.comment_reply USING btree (published_at DESC);


--
-- Name: idx_comment_reply_recipient; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_reply_recipient ON public.comment_reply USING btree (recipient_id);


--
-- Name: idx_comment_report_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_report_published ON public.comment_report USING btree (published_at DESC);


--
-- Name: idx_comment_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_comment_score ON public.comment USING btree (score DESC);


--
-- Name: idx_community_actions_became_moderator; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_became_moderator ON public.community_actions USING btree (became_moderator_at) WHERE (became_moderator_at IS NOT NULL);


--
-- Name: idx_community_actions_became_moderator_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_became_moderator_not_null ON public.community_actions USING btree (person_id, community_id) WHERE (became_moderator_at IS NOT NULL);


--
-- Name: idx_community_actions_blocked_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_blocked_not_null ON public.community_actions USING btree (person_id, community_id) WHERE (blocked_at IS NOT NULL);


--
-- Name: idx_community_actions_community; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_community ON public.community_actions USING btree (community_id);


--
-- Name: idx_community_actions_followed; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_followed ON public.community_actions USING btree (followed_at) WHERE (followed_at IS NOT NULL);


--
-- Name: idx_community_actions_followed_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_followed_not_null ON public.community_actions USING btree (person_id, community_id) WHERE ((followed_at IS NOT NULL) OR (follow_state IS NOT NULL));


--
-- Name: idx_community_actions_person; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_person ON public.community_actions USING btree (person_id);


--
-- Name: idx_community_actions_received_ban_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_actions_received_ban_not_null ON public.community_actions USING btree (person_id, community_id) WHERE (received_ban_at IS NOT NULL);


--
-- Name: idx_community_comments; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_comments ON public.community USING btree (comments DESC, id DESC);


--
-- Name: idx_community_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_hot ON public.community USING btree (hot_rank DESC, id DESC);


--
-- Name: idx_community_lower_actor_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE UNIQUE INDEX idx_community_lower_actor_id ON public.community USING btree (lower((ap_id)::text));


--
-- Name: idx_community_lower_name; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_lower_name ON public.community USING btree (lower((name)::text) DESC, id DESC);


--
-- Name: idx_community_nonzero_hotrank; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_nonzero_hotrank ON public.community USING btree (published_at) WHERE (hot_rank <> (0)::double precision);


--
-- Name: idx_community_posts; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_posts ON public.community USING btree (posts DESC, id DESC);


--
-- Name: idx_community_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_published ON public.community USING btree (published_at DESC, id DESC);


--
-- Name: idx_community_random_number; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_random_number ON public.community USING btree (random_number) INCLUDE (local, nsfw) WHERE (NOT (deleted OR removed OR (visibility = 'Private'::public.community_visibility) OR (visibility = 'Unlisted'::public.community_visibility)));


--
-- Name: idx_community_report_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_report_published ON public.community_report USING btree (published_at DESC);


--
-- Name: idx_community_subscribers; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_subscribers ON public.community USING btree (subscribers DESC, id DESC);


--
-- Name: idx_community_subscribers_local; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_subscribers_local ON public.community USING btree (subscribers_local DESC, id DESC);


--
-- Name: idx_community_title; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_title ON public.community USING btree (title DESC, id DESC);


--
-- Name: idx_community_trigram; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_trigram ON public.community USING gin (name public.gin_trgm_ops, title public.gin_trgm_ops);


--
-- Name: idx_community_users_active_day; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_users_active_day ON public.community USING btree (users_active_day DESC, id DESC);


--
-- Name: idx_community_users_active_half_year; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_users_active_half_year ON public.community USING btree (users_active_half_year DESC, id DESC);


--
-- Name: idx_community_users_active_month; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_users_active_month ON public.community USING btree (users_active_month DESC, id DESC);


--
-- Name: idx_community_users_active_week; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_community_users_active_week ON public.community USING btree (users_active_week DESC, id DESC);


--
-- Name: idx_custom_emoji_category; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_custom_emoji_category ON public.custom_emoji USING btree (id, category);


--
-- Name: idx_image_upload_person_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_image_upload_person_id ON public.local_image USING btree (person_id);


--
-- Name: idx_inbox_combined_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_inbox_combined_published ON public.inbox_combined USING btree (published_at DESC, id DESC);


--
-- Name: idx_inbox_combined_published_asc; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_inbox_combined_published_asc ON public.inbox_combined USING btree (public.reverse_timestamp_sort(published_at) DESC, id DESC);


--
-- Name: idx_instance_actions_blocked_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_instance_actions_blocked_not_null ON public.instance_actions USING btree (person_id, instance_id) WHERE (blocked_at IS NOT NULL);


--
-- Name: idx_instance_actions_instance; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_instance_actions_instance ON public.instance_actions USING btree (instance_id);


--
-- Name: idx_instance_actions_person; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_instance_actions_person ON public.instance_actions USING btree (person_id);


--
-- Name: idx_login_token_user_token; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_login_token_user_token ON public.login_token USING btree (user_id, token);


--
-- Name: idx_modlog_combined_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_modlog_combined_published ON public.modlog_combined USING btree (published_at DESC, id DESC);


--
-- Name: idx_multi_community_ap_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_multi_community_ap_id ON public.multi_community USING btree (ap_id);


--
-- Name: idx_multi_community_entry_community_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_multi_community_entry_community_id ON public.multi_community_entry USING btree (community_id);


--
-- Name: idx_multi_community_follow_multi_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_multi_community_follow_multi_id ON public.multi_community_follow USING btree (multi_community_id);


--
-- Name: idx_multi_community_read_from_name; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_multi_community_read_from_name ON public.multi_community USING btree (local) WHERE (local AND (NOT deleted));


--
-- Name: idx_multi_creator_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_multi_creator_id ON public.multi_community USING btree (creator_id);


--
-- Name: idx_path_gist; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_path_gist ON public.comment USING gist (path);


--
-- Name: idx_person_actions_blocked_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_actions_blocked_not_null ON public.person_actions USING btree (person_id, target_id) WHERE (blocked_at IS NOT NULL);


--
-- Name: idx_person_actions_followed_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_actions_followed_not_null ON public.person_actions USING btree (person_id, target_id) WHERE ((followed_at IS NOT NULL) OR (follow_pending IS NOT NULL));


--
-- Name: idx_person_actions_person; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_actions_person ON public.person_actions USING btree (person_id);


--
-- Name: idx_person_actions_target; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_actions_target ON public.person_actions USING btree (target_id);


--
-- Name: idx_person_content_combined_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_content_combined_published ON public.person_content_combined USING btree (published_at DESC, id DESC);


--
-- Name: idx_person_liked_combined; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_liked_combined ON public.person_liked_combined USING btree (person_id);


--
-- Name: idx_person_liked_combined_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_liked_combined_published ON public.person_liked_combined USING btree (liked_at DESC, id DESC);


--
-- Name: idx_person_local_instance; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_local_instance ON public.person USING btree (local DESC, instance_id);


--
-- Name: idx_person_lower_actor_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE UNIQUE INDEX idx_person_lower_actor_id ON public.person USING btree (lower((ap_id)::text));


--
-- Name: idx_person_lower_name; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_lower_name ON public.person USING btree (lower((name)::text));


--
-- Name: idx_person_post_aggregates_person; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_post_aggregates_person ON public.person_post_aggregates USING btree (person_id);


--
-- Name: idx_person_post_aggregates_post; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_post_aggregates_post ON public.person_post_aggregates USING btree (post_id);


--
-- Name: idx_person_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_published ON public.person USING btree (published_at DESC);


--
-- Name: idx_person_saved_combined; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_saved_combined ON public.person_saved_combined USING btree (person_id);


--
-- Name: idx_person_saved_combined_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_saved_combined_published ON public.person_saved_combined USING btree (saved_at DESC, id DESC);


--
-- Name: idx_person_trigram; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_person_trigram ON public.person USING gin (name public.gin_trgm_ops, display_name public.gin_trgm_ops);


--
-- Name: idx_post_actions_hidden_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_hidden_not_null ON public.post_actions USING btree (person_id, post_id) WHERE (hidden_at IS NOT NULL);


--
-- Name: idx_post_actions_like_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_like_score ON public.post_actions USING btree (post_id, like_score, person_id) WHERE (like_score IS NOT NULL);


--
-- Name: idx_post_actions_liked_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_liked_not_null ON public.post_actions USING btree (person_id, post_id) WHERE ((liked_at IS NOT NULL) OR (like_score IS NOT NULL));


--
-- Name: idx_post_actions_on_read_read_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_on_read_read_not_null ON public.post_actions USING btree (person_id, read_at, post_id) WHERE (read_at IS NOT NULL);


--
-- Name: idx_post_actions_person; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_person ON public.post_actions USING btree (person_id);


--
-- Name: idx_post_actions_post; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_post ON public.post_actions USING btree (post_id);


--
-- Name: idx_post_actions_read_comments_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_read_comments_not_null ON public.post_actions USING btree (person_id, post_id) WHERE ((read_comments_at IS NOT NULL) OR (read_comments_amount IS NOT NULL));


--
-- Name: idx_post_actions_read_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_read_not_null ON public.post_actions USING btree (person_id, post_id) WHERE (read_at IS NOT NULL);


--
-- Name: idx_post_actions_saved_not_null; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_actions_saved_not_null ON public.post_actions USING btree (person_id, post_id) WHERE (saved_at IS NOT NULL);


--
-- Name: idx_post_aggregates_community_active; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_active ON public.post_aggregates USING btree (community_id, featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_controversy ON public.post_aggregates USING btree (community_id, featured_local DESC, controversy_rank DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_hot ON public.post_aggregates USING btree (community_id, featured_local DESC, hot_rank DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_most_comments; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_most_comments ON public.post_aggregates USING btree (community_id, featured_local DESC, comments DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_newest_comment_time; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_newest_comment_time ON public.post_aggregates USING btree (community_id, featured_local DESC, newest_comment_time DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_newest_comment_time_necro; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_newest_comment_time_necro ON public.post_aggregates USING btree (community_id, featured_local DESC, newest_comment_time_necro DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_published ON public.post_aggregates USING btree (community_id, featured_local DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_published_asc; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_published_asc ON public.post_aggregates USING btree (community_id, featured_local DESC, public.reverse_timestamp_sort(published) DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_scaled; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_scaled ON public.post_aggregates USING btree (community_id, featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_community_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_community_score ON public.post_aggregates USING btree (community_id, featured_local DESC, score DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_active; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_active ON public.post_aggregates USING btree (community_id, featured_community DESC, hot_rank_active DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_controversy ON public.post_aggregates USING btree (community_id, featured_community DESC, controversy_rank DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_hot ON public.post_aggregates USING btree (community_id, featured_community DESC, hot_rank DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_most_comments; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_most_comments ON public.post_aggregates USING btree (community_id, featured_community DESC, comments DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_newest_comment_time; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time ON public.post_aggregates USING btree (community_id, featured_community DESC, newest_comment_time DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_newest_comment_time_necr; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time_necr ON public.post_aggregates USING btree (community_id, featured_community DESC, newest_comment_time_necro DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_published ON public.post_aggregates USING btree (community_id, featured_community DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_published_asc; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_published_asc ON public.post_aggregates USING btree (community_id, featured_community DESC, public.reverse_timestamp_sort(published) DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_scaled; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_scaled ON public.post_aggregates USING btree (community_id, featured_community DESC, scaled_rank DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_community_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_community_score ON public.post_aggregates USING btree (community_id, featured_community DESC, score DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_active; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_active ON public.post_aggregates USING btree (featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_controversy ON public.post_aggregates USING btree (featured_local DESC, controversy_rank DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_hot ON public.post_aggregates USING btree (featured_local DESC, hot_rank DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_most_comments; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_most_comments ON public.post_aggregates USING btree (featured_local DESC, comments DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_newest_comment_time; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON public.post_aggregates USING btree (featured_local DESC, newest_comment_time DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_newest_comment_time_necro; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time_necro ON public.post_aggregates USING btree (featured_local DESC, newest_comment_time_necro DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_published ON public.post_aggregates USING btree (featured_local DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_published_asc; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_published_asc ON public.post_aggregates USING btree (featured_local DESC, public.reverse_timestamp_sort(published) DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_scaled; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_scaled ON public.post_aggregates USING btree (featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_featured_local_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_featured_local_score ON public.post_aggregates USING btree (featured_local DESC, score DESC, published DESC, post_id DESC);


--
-- Name: idx_post_aggregates_nonzero_hotrank; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_nonzero_hotrank ON public.post_aggregates USING btree (published DESC) WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));


--
-- Name: idx_post_aggregates_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_published ON public.post_aggregates USING btree (published DESC);


--
-- Name: idx_post_aggregates_published_asc; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_aggregates_published_asc ON public.post_aggregates USING btree (public.reverse_timestamp_sort(published) DESC);


--
-- Name: idx_post_community; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community ON public.post USING btree (community_id);


--
-- Name: idx_post_community_active; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_active ON public.post USING btree (community_id, featured_local DESC, hot_rank_active DESC, published_at DESC, id DESC);


--
-- Name: idx_post_community_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_controversy ON public.post USING btree (community_id, featured_local DESC, controversy_rank DESC, id DESC);


--
-- Name: idx_post_community_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_hot ON public.post USING btree (community_id, featured_local DESC, hot_rank DESC, published_at DESC, id DESC);


--
-- Name: idx_post_community_most_comments; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_most_comments ON public.post USING btree (community_id, featured_local DESC, comments DESC, published_at DESC, id DESC);


--
-- Name: idx_post_community_newest_comment_time; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_newest_comment_time ON public.post USING btree (community_id, featured_local DESC, newest_comment_time_at DESC, id DESC);


--
-- Name: idx_post_community_newest_comment_time_necro; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_newest_comment_time_necro ON public.post USING btree (community_id, featured_local DESC, newest_comment_time_necro_at DESC, id DESC);


--
-- Name: idx_post_community_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_published ON public.post USING btree (community_id, published_at DESC, id DESC);


--
-- Name: idx_post_community_scaled; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_scaled ON public.post USING btree (community_id, featured_local DESC, scaled_rank DESC, published_at DESC, id DESC);


--
-- Name: idx_post_community_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_community_score ON public.post USING btree (community_id, featured_local DESC, score DESC, published_at DESC, id DESC);


--
-- Name: idx_post_creator; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_creator ON public.post USING btree (creator_id);


--
-- Name: idx_post_featured_community_active; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_active ON public.post USING btree (community_id, featured_community DESC, hot_rank_active DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_community_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_controversy ON public.post USING btree (community_id, featured_community DESC, controversy_rank DESC, id DESC);


--
-- Name: idx_post_featured_community_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_hot ON public.post USING btree (community_id, featured_community DESC, hot_rank DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_community_most_comments; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_most_comments ON public.post USING btree (community_id, featured_community DESC, comments DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_community_newest_comment_time; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_newest_comment_time ON public.post USING btree (community_id, featured_community DESC, newest_comment_time_at DESC, id DESC);


--
-- Name: idx_post_featured_community_newest_comment_time_necr; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_newest_comment_time_necr ON public.post USING btree (community_id, featured_community DESC, newest_comment_time_necro_at DESC, id DESC);


--
-- Name: idx_post_featured_community_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_published ON public.post USING btree (community_id, featured_community DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_community_scaled; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_scaled ON public.post USING btree (community_id, featured_community DESC, scaled_rank DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_community_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_community_score ON public.post USING btree (community_id, featured_community DESC, score DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_local_active; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_active ON public.post USING btree (featured_local DESC, hot_rank_active DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_local_controversy; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_controversy ON public.post USING btree (featured_local DESC, controversy_rank DESC, id DESC);


--
-- Name: idx_post_featured_local_hot; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_hot ON public.post USING btree (featured_local DESC, hot_rank DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_local_most_comments; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_most_comments ON public.post USING btree (featured_local DESC, comments DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_local_newest_comment_time; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_newest_comment_time ON public.post USING btree (featured_local DESC, newest_comment_time_at DESC, id DESC);


--
-- Name: idx_post_featured_local_newest_comment_time_necro; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_newest_comment_time_necro ON public.post USING btree (featured_local DESC, newest_comment_time_necro_at DESC, id DESC);


--
-- Name: idx_post_featured_local_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_published ON public.post USING btree (featured_local DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_local_scaled; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_scaled ON public.post USING btree (featured_local DESC, scaled_rank DESC, published_at DESC, id DESC);


--
-- Name: idx_post_featured_local_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_featured_local_score ON public.post USING btree (featured_local DESC, score DESC, published_at DESC, id DESC);


--
-- Name: idx_post_language; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_language ON public.post USING btree (language_id);


--
-- Name: idx_post_like_post; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_like_post ON public.post_like USING btree (post_id);


--
-- Name: idx_post_nonzero_hotrank; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_nonzero_hotrank ON public.post USING btree (published_at DESC) WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));


--
-- Name: idx_post_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_published ON public.post USING btree (published_at DESC);


--
-- Name: idx_post_report_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_report_published ON public.post_report USING btree (published_at DESC);


--
-- Name: idx_post_scheduled_publish_time; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_scheduled_publish_time ON public.post USING btree (scheduled_publish_time_at);


--
-- Name: idx_post_trigram; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_trigram ON public.post USING gin (name public.gin_trgm_ops, body public.gin_trgm_ops, alt_text public.gin_trgm_ops);


--
-- Name: idx_post_url; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_url ON public.post USING btree (url);


--
-- Name: idx_post_url_content_type; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_post_url_content_type ON public.post USING gin (url_content_type public.gin_trgm_ops);


--
-- Name: idx_registration_application_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_registration_application_published ON public.registration_application USING btree (published_at DESC);


--
-- Name: idx_report_combined_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_report_combined_published ON public.report_combined USING btree (published_at DESC, id DESC);


--
-- Name: idx_report_combined_published_asc; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_report_combined_published_asc ON public.report_combined USING btree (public.reverse_timestamp_sort(published_at) DESC, id DESC);


--
-- Name: idx_search_combined_published; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_search_combined_published ON public.search_combined USING btree (published_at DESC, id DESC);


--
-- Name: idx_search_combined_published_asc; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_search_combined_published_asc ON public.search_combined USING btree (public.reverse_timestamp_sort(published_at) DESC, id DESC);


--
-- Name: idx_search_combined_score; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_search_combined_score ON public.search_combined USING btree (score DESC, id DESC);


--
-- Name: idx_tagline_published_id; Type: INDEX; Schema: public; Owner: lemmy
--

CREATE INDEX idx_tagline_published_id ON public.tagline USING btree (published_at DESC, id DESC);


--
-- Name: comment_actions_liked_stat; Type: STATISTICS; Schema: public; Owner: lemmy
--

CREATE STATISTICS public.comment_actions_liked_stat ON (liked_at IS NULL), (like_score IS NULL) FROM public.comment_actions;


ALTER STATISTICS public.comment_actions_liked_stat OWNER TO lemmy;

--
-- Name: community_actions_followed_stat; Type: STATISTICS; Schema: public; Owner: lemmy
--

CREATE STATISTICS public.community_actions_followed_stat ON (followed_at IS NULL), (follow_state IS NULL) FROM public.community_actions;


ALTER STATISTICS public.community_actions_followed_stat OWNER TO lemmy;

--
-- Name: person_actions_followed_stat; Type: STATISTICS; Schema: public; Owner: lemmy
--

CREATE STATISTICS public.person_actions_followed_stat ON (followed_at IS NULL), (follow_pending IS NULL) FROM public.person_actions;


ALTER STATISTICS public.person_actions_followed_stat OWNER TO lemmy;

--
-- Name: post_actions_liked_stat; Type: STATISTICS; Schema: public; Owner: lemmy
--

CREATE STATISTICS public.post_actions_liked_stat ON (liked_at IS NULL), (like_score IS NULL), (post_id IS NULL) FROM public.post_actions;


ALTER STATISTICS public.post_actions_liked_stat OWNER TO lemmy;

--
-- Name: post_actions_read_comments_stat; Type: STATISTICS; Schema: public; Owner: lemmy
--

CREATE STATISTICS public.post_actions_read_comments_stat ON (read_comments_at IS NULL), (read_comments_amount IS NULL) FROM public.post_actions;


ALTER STATISTICS public.post_actions_read_comments_stat OWNER TO lemmy;

--
-- Name: comment change_values; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER change_values BEFORE INSERT OR UPDATE ON public.comment FOR EACH ROW EXECUTE FUNCTION r.comment_change_values();


--
-- Name: post change_values; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER change_values BEFORE INSERT ON public.post FOR EACH ROW EXECUTE FUNCTION r.post_change_values();


--
-- Name: private_message change_values; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER change_values BEFORE INSERT ON public.private_message FOR EACH ROW EXECUTE FUNCTION r.private_message_change_values();


--
-- Name: person delete_follow; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_follow BEFORE DELETE ON public.person FOR EACH ROW EXECUTE FUNCTION r.delete_follow_before_person();


--
-- Name: comment delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.comment REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_delete_statement();


--
-- Name: comment_actions delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.comment_actions REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_actions_delete_statement();


--
-- Name: comment_report delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.comment_report REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_report_delete_statement();


--
-- Name: community delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.community REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_delete_statement();


--
-- Name: community_actions delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.community_actions REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_actions_delete_statement();


--
-- Name: community_report delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.community_report REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_report_delete_statement();


--
-- Name: local_user delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.local_user REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.local_user_delete_statement();


--
-- Name: post delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.post REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_delete_statement();


--
-- Name: post_actions delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.post_actions REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_actions_delete_statement();


--
-- Name: post_report delete_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER delete_statement AFTER DELETE ON public.post_report REFERENCING OLD TABLE AS select_old_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_report_delete_statement();


--
-- Name: __diesel_schema_migrations forbid_diesel_cli; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER forbid_diesel_cli BEFORE INSERT OR DELETE OR UPDATE OR TRUNCATE ON public.__diesel_schema_migrations FOR EACH STATEMENT EXECUTE FUNCTION public.forbid_diesel_cli();


--
-- Name: comment_reply inbox_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER inbox_combined AFTER INSERT ON public.comment_reply FOR EACH ROW EXECUTE FUNCTION r.inbox_combined_comment_reply_insert();


--
-- Name: person_comment_mention inbox_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER inbox_combined AFTER INSERT ON public.person_comment_mention FOR EACH ROW EXECUTE FUNCTION r.inbox_combined_person_comment_mention_insert();


--
-- Name: person_post_mention inbox_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER inbox_combined AFTER INSERT ON public.person_post_mention FOR EACH ROW EXECUTE FUNCTION r.inbox_combined_person_post_mention_insert();


--
-- Name: private_message inbox_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER inbox_combined AFTER INSERT ON public.private_message FOR EACH ROW EXECUTE FUNCTION r.inbox_combined_private_message_insert();


--
-- Name: comment insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.comment REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_insert_statement();


--
-- Name: comment_actions insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.comment_actions REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_actions_insert_statement();


--
-- Name: comment_report insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.comment_report REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_report_insert_statement();


--
-- Name: community insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.community REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_insert_statement();


--
-- Name: community_actions insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.community_actions REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_actions_insert_statement();


--
-- Name: community_report insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.community_report REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_report_insert_statement();


--
-- Name: local_user insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.local_user REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.local_user_insert_statement();


--
-- Name: post insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.post REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_insert_statement();


--
-- Name: post_actions insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.post_actions REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_actions_insert_statement();


--
-- Name: post_report insert_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER insert_statement AFTER INSERT ON public.post_report REFERENCING NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_report_insert_statement();


--
-- Name: admin_allow_instance modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.admin_allow_instance FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_admin_allow_instance_insert();


--
-- Name: admin_block_instance modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.admin_block_instance FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_admin_block_instance_insert();


--
-- Name: admin_purge_comment modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.admin_purge_comment FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_admin_purge_comment_insert();


--
-- Name: admin_purge_community modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.admin_purge_community FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_admin_purge_community_insert();


--
-- Name: admin_purge_person modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.admin_purge_person FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_admin_purge_person_insert();


--
-- Name: admin_purge_post modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.admin_purge_post FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_admin_purge_post_insert();


--
-- Name: mod_add modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_add FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_add_insert();


--
-- Name: mod_add_community modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_add_community FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_add_community_insert();


--
-- Name: mod_ban modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_ban FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_ban_insert();


--
-- Name: mod_ban_from_community modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_ban_from_community FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_ban_from_community_insert();


--
-- Name: mod_change_community_visibility modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_change_community_visibility FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_change_community_visibility_insert();


--
-- Name: mod_feature_post modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_feature_post FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_feature_post_insert();


--
-- Name: mod_lock_post modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_lock_post FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_lock_post_insert();


--
-- Name: mod_remove_comment modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_remove_comment FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_remove_comment_insert();


--
-- Name: mod_remove_community modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_remove_community FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_remove_community_insert();


--
-- Name: mod_remove_post modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_remove_post FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_remove_post_insert();


--
-- Name: mod_transfer_community modlog_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER modlog_combined AFTER INSERT ON public.mod_transfer_community FOR EACH ROW EXECUTE FUNCTION r.modlog_combined_mod_transfer_community_insert();


--
-- Name: comment person_content_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER person_content_combined AFTER INSERT ON public.comment FOR EACH ROW EXECUTE FUNCTION r.person_content_combined_comment_insert();


--
-- Name: post person_content_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER person_content_combined AFTER INSERT ON public.post FOR EACH ROW EXECUTE FUNCTION r.person_content_combined_post_insert();


--
-- Name: comment_actions person_liked_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER person_liked_combined AFTER INSERT OR DELETE OR UPDATE OF liked_at ON public.comment_actions FOR EACH ROW EXECUTE FUNCTION r.person_liked_combined_change_values_comment();


--
-- Name: post_actions person_liked_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER person_liked_combined AFTER INSERT OR DELETE OR UPDATE OF liked_at ON public.post_actions FOR EACH ROW EXECUTE FUNCTION r.person_liked_combined_change_values_post();


--
-- Name: comment_actions person_saved_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER person_saved_combined AFTER INSERT OR DELETE OR UPDATE OF saved_at ON public.comment_actions FOR EACH ROW EXECUTE FUNCTION r.person_saved_combined_change_values_comment();


--
-- Name: post_actions person_saved_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER person_saved_combined AFTER INSERT OR DELETE OR UPDATE OF saved_at ON public.post_actions FOR EACH ROW EXECUTE FUNCTION r.person_saved_combined_change_values_post();


--
-- Name: comment_report report_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER report_combined AFTER INSERT ON public.comment_report FOR EACH ROW EXECUTE FUNCTION r.report_combined_comment_report_insert();


--
-- Name: community_report report_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER report_combined AFTER INSERT ON public.community_report FOR EACH ROW EXECUTE FUNCTION r.report_combined_community_report_insert();


--
-- Name: post_report report_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER report_combined AFTER INSERT ON public.post_report FOR EACH ROW EXECUTE FUNCTION r.report_combined_post_report_insert();


--
-- Name: private_message_report report_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER report_combined AFTER INSERT ON public.private_message_report FOR EACH ROW EXECUTE FUNCTION r.report_combined_private_message_report_insert();


--
-- Name: comment_actions require_uplete; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER require_uplete BEFORE DELETE ON public.comment_actions FOR EACH STATEMENT EXECUTE FUNCTION r.require_uplete();


--
-- Name: community_actions require_uplete; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER require_uplete BEFORE DELETE ON public.community_actions FOR EACH STATEMENT EXECUTE FUNCTION r.require_uplete();


--
-- Name: instance_actions require_uplete; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER require_uplete BEFORE DELETE ON public.instance_actions FOR EACH STATEMENT EXECUTE FUNCTION r.require_uplete();


--
-- Name: person_actions require_uplete; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER require_uplete BEFORE DELETE ON public.person_actions FOR EACH STATEMENT EXECUTE FUNCTION r.require_uplete();


--
-- Name: post_actions require_uplete; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER require_uplete BEFORE DELETE ON public.post_actions FOR EACH STATEMENT EXECUTE FUNCTION r.require_uplete();


--
-- Name: comment search_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined AFTER INSERT ON public.comment FOR EACH ROW EXECUTE FUNCTION r.search_combined_comment_insert();


--
-- Name: community search_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined AFTER INSERT ON public.community FOR EACH ROW EXECUTE FUNCTION r.search_combined_community_insert();


--
-- Name: multi_community search_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined AFTER INSERT ON public.multi_community FOR EACH ROW EXECUTE FUNCTION r.search_combined_multi_community_insert();


--
-- Name: person search_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined AFTER INSERT ON public.person FOR EACH ROW EXECUTE FUNCTION r.search_combined_person_insert();


--
-- Name: post search_combined; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined AFTER INSERT ON public.post FOR EACH ROW EXECUTE FUNCTION r.search_combined_post_insert();


--
-- Name: comment search_combined_comment_score; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined_comment_score AFTER UPDATE OF score ON public.comment FOR EACH ROW EXECUTE FUNCTION r.search_combined_comment_score_update();


--
-- Name: community search_combined_community_score; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined_community_score AFTER UPDATE OF users_active_month ON public.community FOR EACH ROW EXECUTE FUNCTION r.search_combined_community_score_update();


--
-- Name: person search_combined_person_score; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined_person_score AFTER UPDATE OF post_score ON public.person FOR EACH ROW EXECUTE FUNCTION r.search_combined_person_score_update();


--
-- Name: post search_combined_post_score; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER search_combined_post_score AFTER UPDATE OF score ON public.post FOR EACH ROW EXECUTE FUNCTION r.search_combined_post_score_update();


--
-- Name: comment update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.comment REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_update_statement();


--
-- Name: comment_actions update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.comment_actions REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_actions_update_statement();


--
-- Name: comment_report update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.comment_report REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.comment_report_update_statement();


--
-- Name: community update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.community REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_update_statement();


--
-- Name: community_actions update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.community_actions REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_actions_update_statement();


--
-- Name: community_report update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.community_report REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.community_report_update_statement();


--
-- Name: local_user update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.local_user REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.local_user_update_statement();


--
-- Name: post update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.post REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_update_statement();


--
-- Name: post_actions update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.post_actions REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_actions_update_statement();


--
-- Name: post_report update_statement; Type: TRIGGER; Schema: public; Owner: lemmy
--

CREATE TRIGGER update_statement AFTER UPDATE ON public.post_report REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows FOR EACH STATEMENT EXECUTE FUNCTION r.post_report_update_statement();


--
-- Name: admin_allow_instance admin_allow_instance_admin_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_allow_instance
    ADD CONSTRAINT admin_allow_instance_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_allow_instance admin_allow_instance_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_allow_instance
    ADD CONSTRAINT admin_allow_instance_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_block_instance admin_block_instance_admin_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_block_instance
    ADD CONSTRAINT admin_block_instance_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_block_instance admin_block_instance_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_block_instance
    ADD CONSTRAINT admin_block_instance_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_purge_comment admin_purge_comment_admin_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_comment
    ADD CONSTRAINT admin_purge_comment_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_purge_comment admin_purge_comment_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_comment
    ADD CONSTRAINT admin_purge_comment_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_purge_community admin_purge_community_admin_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_community
    ADD CONSTRAINT admin_purge_community_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_purge_person admin_purge_person_admin_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_person
    ADD CONSTRAINT admin_purge_person_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_purge_post admin_purge_post_admin_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_post
    ADD CONSTRAINT admin_purge_post_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_purge_post admin_purge_post_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.admin_purge_post
    ADD CONSTRAINT admin_purge_post_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_actions comment_actions_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_actions
    ADD CONSTRAINT comment_actions_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_actions comment_actions_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_actions
    ADD CONSTRAINT comment_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_aggregates comment_aggregates_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_aggregates
    ADD CONSTRAINT comment_aggregates_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment comment_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment
    ADD CONSTRAINT comment_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment comment_language_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment
    ADD CONSTRAINT comment_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id);


--
-- Name: comment_like comment_like_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_like
    ADD CONSTRAINT comment_like_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_like comment_like_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_like
    ADD CONSTRAINT comment_like_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment comment_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment
    ADD CONSTRAINT comment_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_reply comment_reply_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_reply
    ADD CONSTRAINT comment_reply_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_reply comment_reply_recipient_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_reply
    ADD CONSTRAINT comment_reply_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_report comment_report_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_report
    ADD CONSTRAINT comment_report_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_report comment_report_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_report
    ADD CONSTRAINT comment_report_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: comment_report comment_report_resolver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.comment_report
    ADD CONSTRAINT comment_report_resolver_id_fkey FOREIGN KEY (resolver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_actions community_actions_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_actions
    ADD CONSTRAINT community_actions_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_actions community_actions_follow_approver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_actions
    ADD CONSTRAINT community_actions_follow_approver_id_fkey FOREIGN KEY (follow_approver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_actions community_actions_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_actions
    ADD CONSTRAINT community_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community community_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community
    ADD CONSTRAINT community_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_language community_language_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_language
    ADD CONSTRAINT community_language_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_language community_language_language_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_language
    ADD CONSTRAINT community_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_report community_report_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_report
    ADD CONSTRAINT community_report_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_report community_report_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_report
    ADD CONSTRAINT community_report_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: community_report community_report_resolver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.community_report
    ADD CONSTRAINT community_report_resolver_id_fkey FOREIGN KEY (resolver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: custom_emoji_keyword custom_emoji_keyword_custom_emoji_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.custom_emoji_keyword
    ADD CONSTRAINT custom_emoji_keyword_custom_emoji_id_fkey FOREIGN KEY (custom_emoji_id) REFERENCES public.custom_emoji(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: email_verification email_verification_local_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.email_verification
    ADD CONSTRAINT email_verification_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: federation_allowlist federation_allowlist_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.federation_allowlist
    ADD CONSTRAINT federation_allowlist_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: federation_blocklist federation_blocklist_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.federation_blocklist
    ADD CONSTRAINT federation_blocklist_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: federation_queue_state federation_queue_state_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.federation_queue_state
    ADD CONSTRAINT federation_queue_state_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id);


--
-- Name: inbox_combined inbox_combined_comment_reply_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_comment_reply_id_fkey FOREIGN KEY (comment_reply_id) REFERENCES public.comment_reply(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: inbox_combined inbox_combined_person_comment_mention_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_comment_mention_id_fkey FOREIGN KEY (person_comment_mention_id) REFERENCES public.person_comment_mention(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: inbox_combined inbox_combined_person_post_mention_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_post_mention_id_fkey FOREIGN KEY (person_post_mention_id) REFERENCES public.person_post_mention(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: inbox_combined inbox_combined_private_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_private_message_id_fkey FOREIGN KEY (private_message_id) REFERENCES public.private_message(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: instance_actions instance_actions_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.instance_actions
    ADD CONSTRAINT instance_actions_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: instance_actions instance_actions_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.instance_actions
    ADD CONSTRAINT instance_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_image local_image_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_image
    ADD CONSTRAINT local_image_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_image local_image_thumbnail_for_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_image
    ADD CONSTRAINT local_image_thumbnail_for_post_id_fkey FOREIGN KEY (thumbnail_for_post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_site local_site_multi_comm_follower_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_multi_comm_follower_fkey FOREIGN KEY (multi_comm_follower) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_site_rate_limit local_site_rate_limit_local_site_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site_rate_limit
    ADD CONSTRAINT local_site_rate_limit_local_site_id_fkey FOREIGN KEY (local_site_id) REFERENCES public.local_site(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_site local_site_site_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_site_id_fkey FOREIGN KEY (site_id) REFERENCES public.site(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_site local_site_suggested_communities_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_suggested_communities_fkey FOREIGN KEY (suggested_communities) REFERENCES public.multi_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_user_keyword_block local_user_keyword_block_local_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user_keyword_block
    ADD CONSTRAINT local_user_keyword_block_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_user_language local_user_language_language_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user_language
    ADD CONSTRAINT local_user_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_user_language local_user_language_local_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user_language
    ADD CONSTRAINT local_user_language_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: local_user local_user_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: login_token login_token_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.login_token
    ADD CONSTRAINT login_token_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_add_community mod_add_community_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add_community
    ADD CONSTRAINT mod_add_community_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_add_community mod_add_community_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add_community
    ADD CONSTRAINT mod_add_community_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_add_community mod_add_community_other_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add_community
    ADD CONSTRAINT mod_add_community_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_add mod_add_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add
    ADD CONSTRAINT mod_add_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_add mod_add_other_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_add
    ADD CONSTRAINT mod_add_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_ban_from_community mod_ban_from_community_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban_from_community
    ADD CONSTRAINT mod_ban_from_community_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_ban_from_community mod_ban_from_community_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban_from_community
    ADD CONSTRAINT mod_ban_from_community_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_ban_from_community mod_ban_from_community_other_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban_from_community
    ADD CONSTRAINT mod_ban_from_community_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_ban mod_ban_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban
    ADD CONSTRAINT mod_ban_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_ban mod_ban_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban
    ADD CONSTRAINT mod_ban_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_ban mod_ban_other_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_ban
    ADD CONSTRAINT mod_ban_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_change_community_visibility mod_change_community_visibility_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_change_community_visibility
    ADD CONSTRAINT mod_change_community_visibility_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_change_community_visibility mod_change_community_visibility_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_change_community_visibility
    ADD CONSTRAINT mod_change_community_visibility_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_lock_post mod_lock_post_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_lock_post
    ADD CONSTRAINT mod_lock_post_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_lock_post mod_lock_post_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_lock_post
    ADD CONSTRAINT mod_lock_post_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_remove_comment mod_remove_comment_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_comment
    ADD CONSTRAINT mod_remove_comment_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_remove_comment mod_remove_comment_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_comment
    ADD CONSTRAINT mod_remove_comment_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_remove_community mod_remove_community_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_community
    ADD CONSTRAINT mod_remove_community_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_remove_community mod_remove_community_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_community
    ADD CONSTRAINT mod_remove_community_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_remove_post mod_remove_post_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_post
    ADD CONSTRAINT mod_remove_post_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_remove_post mod_remove_post_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_remove_post
    ADD CONSTRAINT mod_remove_post_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_feature_post mod_sticky_post_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_feature_post
    ADD CONSTRAINT mod_sticky_post_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_feature_post mod_sticky_post_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_feature_post
    ADD CONSTRAINT mod_sticky_post_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_transfer_community mod_transfer_community_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_transfer_community
    ADD CONSTRAINT mod_transfer_community_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_transfer_community mod_transfer_community_mod_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_transfer_community
    ADD CONSTRAINT mod_transfer_community_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mod_transfer_community mod_transfer_community_other_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.mod_transfer_community
    ADD CONSTRAINT mod_transfer_community_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_admin_allow_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_allow_instance_id_fkey FOREIGN KEY (admin_allow_instance_id) REFERENCES public.admin_allow_instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_admin_block_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_block_instance_id_fkey FOREIGN KEY (admin_block_instance_id) REFERENCES public.admin_block_instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_admin_purge_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_comment_id_fkey FOREIGN KEY (admin_purge_comment_id) REFERENCES public.admin_purge_comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_admin_purge_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_community_id_fkey FOREIGN KEY (admin_purge_community_id) REFERENCES public.admin_purge_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_admin_purge_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_person_id_fkey FOREIGN KEY (admin_purge_person_id) REFERENCES public.admin_purge_person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_admin_purge_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_post_id_fkey FOREIGN KEY (admin_purge_post_id) REFERENCES public.admin_purge_post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_add_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_community_id_fkey FOREIGN KEY (mod_add_community_id) REFERENCES public.mod_add_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_add_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_id_fkey FOREIGN KEY (mod_add_id) REFERENCES public.mod_add(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_ban_from_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_from_community_id_fkey FOREIGN KEY (mod_ban_from_community_id) REFERENCES public.mod_ban_from_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_ban_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_id_fkey FOREIGN KEY (mod_ban_id) REFERENCES public.mod_ban(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_change_community_visibility_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_change_community_visibility_id_fkey FOREIGN KEY (mod_change_community_visibility_id) REFERENCES public.mod_change_community_visibility(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_feature_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_feature_post_id_fkey FOREIGN KEY (mod_feature_post_id) REFERENCES public.mod_feature_post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_lock_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_fkey FOREIGN KEY (mod_lock_post_id) REFERENCES public.mod_lock_post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_remove_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_comment_id_fkey FOREIGN KEY (mod_remove_comment_id) REFERENCES public.mod_remove_comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_remove_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_community_id_fkey FOREIGN KEY (mod_remove_community_id) REFERENCES public.mod_remove_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_remove_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_fkey FOREIGN KEY (mod_remove_post_id) REFERENCES public.mod_remove_post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: modlog_combined modlog_combined_mod_transfer_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_transfer_community_id_fkey FOREIGN KEY (mod_transfer_community_id) REFERENCES public.mod_transfer_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: multi_community multi_community_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community
    ADD CONSTRAINT multi_community_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: multi_community_entry multi_community_entry_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community_entry
    ADD CONSTRAINT multi_community_entry_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: multi_community_entry multi_community_entry_multi_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community_entry
    ADD CONSTRAINT multi_community_entry_multi_community_id_fkey FOREIGN KEY (multi_community_id) REFERENCES public.multi_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: multi_community_follow multi_community_follow_multi_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community_follow
    ADD CONSTRAINT multi_community_follow_multi_community_id_fkey FOREIGN KEY (multi_community_id) REFERENCES public.multi_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: multi_community_follow multi_community_follow_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community_follow
    ADD CONSTRAINT multi_community_follow_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: multi_community multi_community_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.multi_community
    ADD CONSTRAINT multi_community_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: oauth_account oauth_account_local_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: oauth_account oauth_account_oauth_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_oauth_provider_id_fkey FOREIGN KEY (oauth_provider_id) REFERENCES public.oauth_provider(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- Name: password_reset_request password_reset_request_local_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.password_reset_request
    ADD CONSTRAINT password_reset_request_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_actions person_actions_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_actions
    ADD CONSTRAINT person_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_actions person_actions_target_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_actions
    ADD CONSTRAINT person_actions_target_id_fkey FOREIGN KEY (target_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_content_combined person_content_combined_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_content_combined person_content_combined_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person person_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person
    ADD CONSTRAINT person_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_liked_combined person_liked_combined_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_liked_combined person_liked_combined_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_liked_combined person_liked_combined_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_comment_mention person_mention_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_comment_mention
    ADD CONSTRAINT person_mention_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_comment_mention person_mention_recipient_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_comment_mention
    ADD CONSTRAINT person_mention_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_post_aggregates person_post_aggregates_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_aggregates
    ADD CONSTRAINT person_post_aggregates_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_post_aggregates person_post_aggregates_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_aggregates
    ADD CONSTRAINT person_post_aggregates_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_post_mention person_post_mention_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_post_mention person_post_mention_recipient_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_saved_combined person_saved_combined_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_saved_combined person_saved_combined_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: person_saved_combined person_saved_combined_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_actions post_actions_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_actions
    ADD CONSTRAINT post_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_actions post_actions_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_actions
    ADD CONSTRAINT post_actions_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_aggregates post_aggregates_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_aggregates
    ADD CONSTRAINT post_aggregates_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_aggregates post_aggregates_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_aggregates
    ADD CONSTRAINT post_aggregates_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_aggregates post_aggregates_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_aggregates
    ADD CONSTRAINT post_aggregates_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_aggregates post_aggregates_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_aggregates
    ADD CONSTRAINT post_aggregates_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post post_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post post_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post post_language_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id);


--
-- Name: post_like post_like_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_like
    ADD CONSTRAINT post_like_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_like post_like_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_like
    ADD CONSTRAINT post_like_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_read post_read_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_read
    ADD CONSTRAINT post_read_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_read post_read_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_read
    ADD CONSTRAINT post_read_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_report post_report_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_report post_report_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_report post_report_resolver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_resolver_id_fkey FOREIGN KEY (resolver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_tag post_tag_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_tag
    ADD CONSTRAINT post_tag_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: post_tag post_tag_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.post_tag
    ADD CONSTRAINT post_tag_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tag(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_message private_message_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message
    ADD CONSTRAINT private_message_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_message private_message_recipient_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message
    ADD CONSTRAINT private_message_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_message_report private_message_report_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message_report
    ADD CONSTRAINT private_message_report_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_message_report private_message_report_private_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message_report
    ADD CONSTRAINT private_message_report_private_message_id_fkey FOREIGN KEY (private_message_id) REFERENCES public.private_message(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_message_report private_message_report_resolver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.private_message_report
    ADD CONSTRAINT private_message_report_resolver_id_fkey FOREIGN KEY (resolver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: registration_application registration_application_admin_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.registration_application
    ADD CONSTRAINT registration_application_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: registration_application registration_application_local_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.registration_application
    ADD CONSTRAINT registration_application_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: report_combined report_combined_comment_report_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_comment_report_id_fkey FOREIGN KEY (comment_report_id) REFERENCES public.comment_report(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: report_combined report_combined_community_report_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_community_report_id_fkey FOREIGN KEY (community_report_id) REFERENCES public.community_report(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: report_combined report_combined_post_report_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_post_report_id_fkey FOREIGN KEY (post_report_id) REFERENCES public.post_report(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: report_combined report_combined_private_message_report_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_private_message_report_id_fkey FOREIGN KEY (private_message_report_id) REFERENCES public.private_message_report(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: search_combined search_combined_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comment(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: search_combined search_combined_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: search_combined search_combined_multi_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_multi_community_id_fkey FOREIGN KEY (multi_community_id) REFERENCES public.multi_community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: search_combined search_combined_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: search_combined search_combined_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site site_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site
    ADD CONSTRAINT site_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_language site_language_language_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site_language
    ADD CONSTRAINT site_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_language site_language_site_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.site_language
    ADD CONSTRAINT site_language_site_id_fkey FOREIGN KEY (site_id) REFERENCES public.site(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: tag tag_community_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: lemmy
--

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_community_id_fkey FOREIGN KEY (community_id) REFERENCES public.community(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: lemmy
--

REVOKE USAGE ON SCHEMA public FROM PUBLIC;


--
-- PostgreSQL database dump complete
--

