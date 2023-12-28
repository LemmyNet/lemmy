-- This sets up the `r` schema, which contains things that can be safely dropped and replaced instead of being
-- changed using migrations.
--
-- Statements in this file may not create or modify things outside of the `r` schema (indicated by the `r.` prefix),
-- except for these things, which are associated with something other than a schema (usually a table):
--   * A trigger if the function name after `EXECUTE FUNCTION` is in `r` (dropping `r` drops the trigger)
--
-- The default schema is not temporarily set to `r` because it would not affect some things (such as triggers) which
-- makes it hard to tell if the rule above is being followed.
--
-- If you add something here that depends on something (such as a table) created in a new migration, then down.sql must use
-- `CASCADE` when dropping it. This doesn't need to be fixed in old migrations because the "replaceable-schema" migration
-- runs `DROP SCHEMA IF EXISTS r CASCADE` in down.sql.
BEGIN;

DROP SCHEMA IF EXISTS r CASCADE;

CREATE SCHEMA r;

-- Rank calculations
CREATE FUNCTION r.controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS float
    LANGUAGE plpgsql
    IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    IF downvotes <= 0 OR upvotes <= 0 THEN
        RETURN 0;
    ELSE
        RETURN (upvotes + downvotes) * CASE WHEN upvotes > downvotes THEN
            downvotes::float / upvotes::float
        ELSE
            upvotes::float / downvotes::float
        END;
    END IF;
END
$$;

-- Selects both old and new rows in a trigger and allows using `sum(count_diff)` to get the number to add to a count
CREATE FUNCTION r.combine_transition_tables ()
    RETURNS SETOF record
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        -1 AS count_diff,
        *
    FROM
        old_table
    UNION ALL
    SELECT
        1 AS count_diff,
        *
    FROM
        new_table;
END
$$;

-- Creates triggers for all operation types, which can't be 1 trigger when transition tables are used
CREATE PROCEDURE r.create_triggers (table_name text, function_name text)
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER %2$s_insert AFTER INSERT ON %1$s REFERENCING NEW TABLE AS new_table FOR EACH STATEMENT EXECUTE FUNCTION r.%2$s ();', table_name, function_name);
    EXECUTE format('CREATE TRIGGER %2$s_delete AFTER DELETE ON %1$s REFERENCING OLD TABLE AS old_table FOR EACH STATEMENT EXECUTE FUNCTION r.%2$s ();', table_name, function_name);
    EXECUTE format('CREATE TRIGGER %2$s_update AFTER UPDATE ON %1$s REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table FOR EACH STATEMENT EXECUTE FUNCTION r.%2$s ();', table_name, function_name);
END
    $$;

-- Define functions
CREATE FUNCTION r.creator_id_from_post_aggregates (agg post_aggregates)
    RETURNS int RETURN agg.creator_id;

CREATE FUNCTION r.creator_id_from_comment_aggregates (agg comment_aggregates)
    RETURNS int RETURN (
        SELECT
            creator_id
        FROM
            comment
        WHERE
            comment.id = agg.comment_id LIMIT 1
);

-- Create triggers for both post and comments
CREATE PROCEDURE r.post_or_comment (thing_type text)
LANGUAGE plpgsql
AS $a$
BEGIN
    EXECUTE replace($b$
        -- When a thing is removed, resolve its reports
        CREATE FUNCTION r.resolve_reports_when_thing_removed ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                UPDATE
                    thing_report
                SET
                    resolved = TRUE, resolver_id = first_removal.mod_person_id, updated = first_removal.when_ FROM (
                        SELECT
                            thing_id, min(when_) AS when_ FROM new_removal
                        WHERE
                            new_removal.removed GROUP BY thing_id) AS first_removal
                        WHERE
                            report.thing_id = first_removal.thing_id
                            AND NOT report.resolved
                            AND COALESCE(report.updated < first_removal.when_, TRUE);
                RETURN NULL;
            END $$;
    CREATE TRIGGER resolve_reports
        AFTER INSERT ON mod_remove_thing REFERENCING NEW TABLE AS new_removal
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.resolve_reports_when_thing_removed ( );
        -- When a thing gets a vote, update its aggregates and its creator's aggregates
        CREATE FUNCTION r.thing_aggregates_from_like ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                WITH thing_diff AS (
                    UPDATE
                        thing_aggregates AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes,
                        upvotes = a.upvotes + diff.upvotes,
                        downvotes = a.downvotes + diff.downvotes,
                        controversy_rank = controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            thing_id,
                            sum(count_diff) FILTER (WHERE score = 1) AS upvotes,
                            sum(count_diff) FILTER (WHERE score != 1) AS downvotes
                        FROM
                            r.combine_transition_tables ()
                        GROUP BY
                            thing_id) AS diff
                    WHERE
                        a.thing_id = diff.thing_id
                    RETURNING
                        creator_id_from_thing_aggregates (a.*) AS creator_id,
                        diff.upvotes - diff.downvotes AS score)
            UPDATE
                person_aggregates AS a
            SET
                thing_score = a.thing_score + diff.score
            FROM (
                SELECT
                    creator_id,
                    sum(score) AS score
                FROM
                    target_diff
                GROUP BY
                    creator_id) AS diff
        WHERE
            a.person_id = diff.creator_id;
                RETURN NULL;
            END $$;
    CREATE TRIGGER aggregates
        AFTER INSERT OR DELETE OR UPDATE OF score ON thing_like REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.thing_aggregates_from_like ( );
        $b$,
        'thing',
        thing_type);
END
$a$;

CALL r.post_or_comment ('post');

CALL r.post_or_comment ('comment');

-- Create triggers that update counts in parent aggregates
CREATE FUNCTION r.parent_aggregates_from_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    WITH comment_group AS (
        SELECT
            post_id,
            creator_id,
            local,
            sum(count_diff) AS comments,
        FROM
            combine_transition_tables ()
        WHERE
            NOT (deleted
                OR removed)
        GROUP BY
            GROUPING SETS (post_id,
                creator_id,
                local)
),
unused_person_aggregates_update_result AS (
    UPDATE
        person_aggregates AS a
    SET
        comment_count = a.comment_count + comment_group.comments
    FROM
        comment_group
    WHERE
        a.person_id = comment_group.creator_id
),
unused_site_aggregates_update_result AS (
    UPDATE
        site_aggregates AS a
    SET
        comments = a.comments + comment_group.comments
    FROM
        comment_group
    WHERE
        comment_group.local
),
post_diff AS (
    UPDATE
        post_aggregates AS a
    SET
        comments = a.comments + comment_group.comments,
        newest_comment_time = GREATEST (a.newest_comment_time, (
                SELECT
                    max(published)
                FROM new_table AS new_comment
                WHERE
                    a.post_id = new_comment.post_id)
            LIMIT 1),
        newest_comment_time_necro = GREATEST (a.newest_comment_time_necro, (
                SELECT
                    max(published)
                FROM new_table AS new_comment
                WHERE
                    a.post_id = new_comment.post_id
                    -- Ignore comments from the post's creator
                    AND a.creator_id != new_comment.creator_id
                    -- Ignore comments on old posts
                    AND a.published > (new_comment.published - '2 days'::interval)
                LIMIT 1))
    FROM
        comment_group,
        LATERAL (
            SELECT
                *
            FROM
                post
            WHERE
                a.post_id = post.id
            LIMIT 1) AS post
    WHERE
        a.post_id = comment_group.post_id
    RETURNING
        a.community_id,
        diff.comments,
        NOT (post.deleted
            OR post.removed) AS include_in_community_aggregates)
UPDATE
    community_aggregates AS a
SET
    comments = a.comments + diff.comments
FROM (
    SELECT
        community_id,
        sum(comments)
    FROM
        post_diff
    WHERE
        post_diff.include_in_community_aggregates
    GROUP BY
        community_id) AS diff
WHERE
    a.community_id = diff.community_id;
        RETURN NULL;
END
$$;

CREATE TRIGGER parent_aggregates
    AFTER INSERT OR DELETE OR UPDATE OF deleted,
    removed ON comment REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.parent_aggregates_from_comment ();

CREATE FUNCTION r.parent_aggregates_from_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    WITH post_group AS (
        SELECT
            community_id,
            creator_id,
            local,
            sum(count_diff) AS posts,
        FROM
            combine_transition_tables ()
        WHERE
            NOT (deleted
                OR removed)
        GROUP BY
            GROUPING SETS (community_id,
                creator_id,
                local)
),
unused_person_aggregates_update_result AS (
    UPDATE
        person_aggregates AS a
    SET
        post_count = a.post_count + post_group.posts
    FROM
        post_group
    WHERE
        a.person_id = post_group.creator_id
),
unused_site_aggregates_update_result AS (
    UPDATE
        site_aggregates AS a
    SET
        posts = a.posts + post_group.posts
    FROM
        post_group
    WHERE
        post_group.local)
UPDATE
    community_aggregates AS a
SET
    posts = a.posts + post_group.posts
FROM
    post_group
WHERE
    a.community_id = post_group.community_id;
    RETURN NULL;
END
$$;

CREATE TRIGGER parent_aggregates
    AFTER INSERT OR DELETE OR UPDATE OF deleted,
    removed ON comment REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.parent_aggregates_from_comment ();

CREATE FUNCTION site_aggregates_from_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates AS a
    SET
        communities = a.communities + diff.communities
    FROM (
        SELECT
            sum(change_diff) AS communities
        FROM
            combine_transition_tables ()
        WHERE
            local
            AND NOT (deleted
                OR removed)) AS diff;
    RETURN NULL;
$$;

CREATE TRIGGER site_aggregates
    AFTER INSERT OR DELETE OR UPDATE OF deleted,
    removed ON community REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.site_aggregates_from_community ();

CREATE FUNCTION site_aggregates_from_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates AS a
    SET
        users = a.users + diff.users
    FROM (
        SELECT
            sum(change_diff) AS users
        FROM
            combine_transition_tables ()
        WHERE
            local) AS diff;
    RETURN NULL;
$$;

CREATE TRIGGER site_aggregates
    AFTER INSERT OR DELETE ON person REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.site_aggregates_from_person ();

-- For community_aggregates.comments, don't include comments of deleted or removed posts
CREATE FUNCTION r.update_comment_count_from_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community_aggregates AS a
    SET
        comments = a.comments + diff.comments
    FROM (
        SELECT
            old_post.community_id,
            sum((
                CASE WHEN new_post.deleted
                    AND new_post.removed THEN
                    -1
                ELSE
                    1
                END) * post_aggregates.comments) AS comments
        FROM
            new_post
            INNER JOIN old_post ON new_post.id = old_post.id
                AND (new_post.deleted
                    AND new_post.removed) != (old_post.deleted
                    AND old_post.removed),
                LATERAL (
                    SELECT
                        *
                    FROM
                        post_aggregates
                WHERE
                    post_id = new_post.id
                LIMIT 1) AS post_aggregates
        GROUP BY
            old_post.community_id) AS diff
WHERE
    a.community_id = diff.community_id;
    RETURN NULL;
$$;

CREATE TRIGGER comment_count
    AFTER UPDATE OF deleted,
    removed ON post REFERENCING OLD TABLE AS old_post NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.update_comment_count_from_post ();

-- Count subscribers for local communities
CREATE FUNCTION r.community_aggregates_from_subscriber ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community_aggregates AS a
    SET
        subscriber = a.subscribers + diff.subscribers
    FROM (
        SELECT
            community_id,
            sum(count_diff) AS subscribers
        FROM
            combine_transition_tables ()
        WHERE (
            SELECT
                local
            FROM
                community
            WHERE
                community.id = community_id
            LIMIT 1)
    GROUP BY
        community_id) AS diff
WHERE
    a.community_id = diff.community_id;
    RETURN NULL;
END
$$;

CREATE TRIGGER community_aggregates
    AFTER INSERT OR DELETE ON community_follower REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.community_aggregates_from_subscriber ();

-- These triggers create and update rows in each aggregates table to match its associated table's rows.
-- Deleting rows and updating IDs are already handled by `CASCADE` in foreign key constraints.
CREATE FUNCTION r.comment_aggregates_from_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO comment_aggregates (comment_id, published)
    SELECT
        id,
        published
    FROM
        new_comment;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON comment REFERENCING NEW TABLE AS new_comment
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.comment_aggregates_from_comment ();

CREATE FUNCTION r.community_aggregates_from_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO community_aggregates (community_id, published)
    SELECT
        community_id,
        published
    FROM
        new_community;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON community REFERENCING NEW TABLE AS new_community
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.community_aggregates_from_community ();

CREATE FUNCTION r.person_aggregates_from_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO person_aggregates (person_id)
    SELECT
        id,
    FROM
        new_person;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON person REFERENCING NEW TABLE AS new_person
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.person_aggregates_from_person ();

CREATE FUNCTION r.post_aggregates_from_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id, featured_community, featured_local)
    SELECT
        new_post.id,
        new_post.published,
        new_post.published,
        new_post.published,
        new_post.community_id,
        new_post.creator_id,
        community.instance_id,
        new_post.featured_community,
        new_post.featured_local
    FROM
        new_post,
        LATERAL (
            SELECT
                *
            FROM
                community
            WHERE
                community.id = new_post.community_id
            LIMIT 1) AS community,
    ON CONFLICT
        DO UPDATE SET
            featured_community = excluded.featured_community,
            featured_local = excluded.featured_local;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT OR UPDATE OF featured_community,
    featured_local ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.post_aggregates_from_post ();

CREATE FUNCTION r.site_aggregates_from_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- we only ever want to have a single value in site_aggregate because the site_aggregate triggers update all rows in that table.
    -- a cleaner check would be to insert it for the local_site but that would break assumptions at least in the tests
    IF NOT EXISTS (
        SELECT
            1
        FROM
            site_aggregates) THEN
    INSERT INTO site_aggregates (site_id)
        VALUES (NEW.id);
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON site
    FOR EACH ROW
    EXECUTE FUNCTION r.site_aggregates_from_site ();

COMMIT;

