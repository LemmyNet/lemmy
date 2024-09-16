DROP SCHEMA IF EXISTS r CASCADE;

DROP INDEX idx_site_aggregates_1_row_only;

CREATE FUNCTION comment_aggregates_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO comment_aggregates (comment_id, published)
            VALUES (NEW.id, NEW.published);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM comment_aggregates
        WHERE comment_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION comment_aggregates_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            comment_aggregates ca
        SET
            score = score + NEW.score,
            upvotes = CASE WHEN NEW.score = 1 THEN
                upvotes + 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN NEW.score = - 1 THEN
                downvotes + 1
            ELSE
                downvotes
            END,
            controversy_rank = controversy_rank (ca.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, ca.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
        WHERE
            ca.comment_id = NEW.comment_id;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Join to comment because that comment may not exist anymore
        UPDATE
            comment_aggregates ca
        SET
            score = score - OLD.score,
            upvotes = CASE WHEN OLD.score = 1 THEN
                upvotes - 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN OLD.score = - 1 THEN
                downvotes - 1
            ELSE
                downvotes
            END,
            controversy_rank = controversy_rank (ca.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, ca.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
        FROM
            comment c
        WHERE
            ca.comment_id = c.id
            AND ca.comment_id = OLD.comment_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION community_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments + 1
        FROM
            post p
        WHERE
            p.id = NEW.post_id
            AND ca.community_id = p.community_id;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments - 1
        FROM
            post p
        WHERE
            p.id = OLD.post_id
            AND ca.community_id = p.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION community_aggregates_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO community_aggregates (community_id, published)
            VALUES (NEW.id, NEW.published);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM community_aggregates
        WHERE community_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION community_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            community_aggregates
        SET
            posts = posts + 1
        WHERE
            community_id = NEW.community_id;
        IF (TG_OP = 'UPDATE') THEN
            -- Post was restored, so restore comment counts as well
            UPDATE
                community_aggregates ca
            SET
                posts = coalesce(cd.posts, 0),
                comments = coalesce(cd.comments, 0)
            FROM (
                SELECT
                    c.id,
                    count(DISTINCT p.id) AS posts,
                    count(DISTINCT ct.id) AS comments
                FROM
                    community c
                LEFT JOIN post p ON c.id = p.community_id
                    AND p.deleted = 'f'
                    AND p.removed = 'f'
            LEFT JOIN comment ct ON p.id = ct.post_id
                AND ct.deleted = 'f'
                AND ct.removed = 'f'
        WHERE
            c.id = NEW.community_id
        GROUP BY
            c.id) cd
        WHERE
            ca.community_id = NEW.community_id;
        END IF;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            community_aggregates
        SET
            posts = posts - 1
        WHERE
            community_id = OLD.community_id;
        -- Update the counts if the post got deleted
        UPDATE
            community_aggregates ca
        SET
            posts = coalesce(cd.posts, 0),
            comments = coalesce(cd.comments, 0)
        FROM (
            SELECT
                c.id,
                count(DISTINCT p.id) AS posts,
                count(DISTINCT ct.id) AS comments
            FROM
                community c
            LEFT JOIN post p ON c.id = p.community_id
                AND p.deleted = 'f'
                AND p.removed = 'f'
        LEFT JOIN comment ct ON p.id = ct.post_id
            AND ct.deleted = 'f'
            AND ct.removed = 'f'
    WHERE
        c.id = OLD.community_id
    GROUP BY
        c.id) cd
    WHERE
        ca.community_id = OLD.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION community_aggregates_post_count_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community_aggregates
    SET
        posts = posts + post_group.count
    FROM (
        SELECT
            community_id,
            count(*)
        FROM
            new_post
        GROUP BY
            community_id) post_group
WHERE
    community_aggregates.community_id = post_group.community_id;
    RETURN NULL;
END
$$;

CREATE FUNCTION community_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates ca
        SET
            subscribers = subscribers + community.local::int,
            subscribers_local = subscribers_local + person.local::int
        FROM
            community
        LEFT JOIN person ON person.id = NEW.person_id
    WHERE
        community.id = NEW.community_id
            AND community.id = ca.community_id
            AND person.local IS NOT NULL;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates ca
        SET
            subscribers = subscribers - community.local::int,
            subscribers_local = subscribers_local - person.local::int
        FROM
            community
        LEFT JOIN person ON person.id = OLD.person_id
    WHERE
        community.id = OLD.community_id
            AND community.id = ca.community_id
            AND person.local IS NOT NULL;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION delete_follow_before_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM community_follower AS c
    WHERE c.person_id = OLD.id;
    RETURN OLD;
END;
$$;

CREATE FUNCTION person_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count - 1
        WHERE
            person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION person_aggregates_comment_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            person_aggregates ua
        SET
            comment_score = comment_score + NEW.score
        FROM
            comment c
        WHERE
            ua.person_id = c.creator_id
            AND c.id = NEW.comment_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates ua
        SET
            comment_score = comment_score - OLD.score
        FROM
            comment c
        WHERE
            ua.person_id = c.creator_id
            AND c.id = OLD.comment_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION person_aggregates_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO person_aggregates (person_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM person_aggregates
        WHERE person_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION person_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count - 1
        WHERE
            person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION person_aggregates_post_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        person_aggregates
    SET
        post_count = post_count + post_group.count
    FROM (
        SELECT
            creator_id,
            count(*)
        FROM
            new_post
        GROUP BY
            creator_id) post_group
WHERE
    person_aggregates.person_id = post_group.creator_id;
    RETURN NULL;
END
$$;

CREATE FUNCTION person_aggregates_post_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            person_aggregates ua
        SET
            post_score = post_score + NEW.score
        FROM
            post p
        WHERE
            ua.person_id = p.creator_id
            AND p.id = NEW.post_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates ua
        SET
            post_score = post_score - OLD.score
        FROM
            post p
        WHERE
            ua.person_id = p.creator_id
            AND p.id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION post_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Check for post existence - it may not exist anymore
    IF TG_OP = 'INSERT' OR EXISTS (
        SELECT
            1
        FROM
            post p
        WHERE
            p.id = OLD.post_id) THEN
        IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
            UPDATE
                post_aggregates pa
            SET
                comments = comments + 1
            WHERE
                pa.post_id = NEW.post_id;
        ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
            UPDATE
                post_aggregates pa
            SET
                comments = comments - 1
            WHERE
                pa.post_id = OLD.post_id;
        END IF;
    END IF;
    IF TG_OP = 'INSERT' THEN
        UPDATE
            post_aggregates pa
        SET
            newest_comment_time = NEW.published
        WHERE
            pa.post_id = NEW.post_id;
        -- A 2 day necro-bump limit
        UPDATE
            post_aggregates pa
        SET
            newest_comment_time_necro = NEW.published
        FROM
            post p
        WHERE
            pa.post_id = p.id
            AND pa.post_id = NEW.post_id
            -- Fix issue with being able to necro-bump your own post
            AND NEW.creator_id != p.creator_id
            AND pa.published > ('now'::timestamp - '2 days'::interval);
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION post_aggregates_featured_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_aggregates pa
    SET
        featured_community = NEW.featured_community
    WHERE
        pa.post_id = NEW.id;
    RETURN NULL;
END
$$;

CREATE FUNCTION post_aggregates_featured_local ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_aggregates pa
    SET
        featured_local = NEW.featured_local
    WHERE
        pa.post_id = NEW.id;
    RETURN NULL;
END
$$;

CREATE FUNCTION post_aggregates_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id)
    SELECT
        id,
        published,
        published,
        published,
        community_id,
        creator_id,
        (
            SELECT
                community.instance_id
            FROM
                community
            WHERE
                community.id = community_id
            LIMIT 1)
FROM
    new_post;
    RETURN NULL;
END
$$;

CREATE FUNCTION post_aggregates_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            post_aggregates pa
        SET
            score = score + NEW.score,
            upvotes = CASE WHEN NEW.score = 1 THEN
                upvotes + 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN NEW.score = - 1 THEN
                downvotes + 1
            ELSE
                downvotes
            END,
            controversy_rank = controversy_rank (pa.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, pa.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
        WHERE
            pa.post_id = NEW.post_id;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Join to post because that post may not exist anymore
        UPDATE
            post_aggregates pa
        SET
            score = score - OLD.score,
            upvotes = CASE WHEN OLD.score = 1 THEN
                upvotes - 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN OLD.score = - 1 THEN
                downvotes - 1
            ELSE
                downvotes
            END,
            controversy_rank = controversy_rank (pa.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, pa.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
        FROM
            post p
        WHERE
            pa.post_id = p.id
            AND pa.post_id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_comment_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            site_aggregates sa
        SET
            comments = comments - 1
        FROM
            site s
        WHERE
            sa.site_id = s.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_comment_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            site_aggregates sa
        SET
            comments = comments + 1
        FROM
            site s
        WHERE
            sa.site_id = s.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_community_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            site_aggregates sa
        SET
            communities = communities - 1
        FROM
            site s
        WHERE
            sa.site_id = s.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_community_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            site_aggregates sa
        SET
            communities = communities + 1
        FROM
            site s
        WHERE
            sa.site_id = s.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_person_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Join to site since the creator might not be there anymore
    UPDATE
        site_aggregates sa
    SET
        users = users - 1
    FROM
        site s
    WHERE
        sa.site_id = s.id;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_person_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates
    SET
        users = users + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_post_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            site_aggregates sa
        SET
            posts = posts - 1
        FROM
            site s
        WHERE
            sa.site_id = s.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_post_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates sa
    SET
        posts = posts + (
            SELECT
                count(*)
            FROM
                new_post)
    FROM
        site s
    WHERE
        sa.site_id = s.id;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_post_update ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            site_aggregates sa
        SET
            posts = posts + 1
        FROM
            site s
        WHERE
            sa.site_id = s.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- we only ever want to have a single value in site_aggregate because the site_aggregate triggers update all rows in that table.
    -- a cleaner check would be to insert it for the local_site but that would break assumptions at least in the tests
    IF (TG_OP = 'INSERT') AND NOT EXISTS (
    SELECT
        *
    FROM
        site_aggregates
    LIMIT 1) THEN
        INSERT INTO site_aggregates (site_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM site_aggregates
        WHERE site_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION was_removed_or_deleted (tg_op text, old record, new record)
    RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        RETURN FALSE;
    END IF;
    IF (TG_OP = 'DELETE' AND OLD.deleted = 'f' AND OLD.removed = 'f') THEN
        RETURN TRUE;
    END IF;
    RETURN TG_OP = 'UPDATE'
        AND OLD.deleted = 'f'
        AND OLD.removed = 'f'
        AND (NEW.deleted = 't'
            OR NEW.removed = 't');
END
$$;

CREATE FUNCTION was_restored_or_created (tg_op text, old record, new record)
    RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        RETURN FALSE;
    END IF;
    IF (TG_OP = 'INSERT') THEN
        RETURN TRUE;
    END IF;
    RETURN TG_OP = 'UPDATE'
        AND NEW.deleted = 'f'
        AND NEW.removed = 'f'
        AND (OLD.deleted = 't'
            OR OLD.removed = 't');
END
$$;

CREATE TRIGGER comment_aggregates_comment
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE FUNCTION comment_aggregates_comment ();

CREATE TRIGGER comment_aggregates_score
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE FUNCTION comment_aggregates_score ();

CREATE TRIGGER community_aggregates_comment_count
    AFTER INSERT OR DELETE OR UPDATE OF removed,
    deleted ON comment
    FOR EACH ROW
    EXECUTE FUNCTION community_aggregates_comment_count ();

CREATE TRIGGER community_aggregates_community
    AFTER INSERT OR DELETE ON community
    FOR EACH ROW
    EXECUTE FUNCTION community_aggregates_community ();

CREATE TRIGGER community_aggregates_post_count
    AFTER DELETE OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    EXECUTE FUNCTION community_aggregates_post_count ();

CREATE TRIGGER community_aggregates_post_count_insert
    AFTER INSERT ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION community_aggregates_post_count_insert ();

CREATE TRIGGER community_aggregates_subscriber_count
    AFTER INSERT OR DELETE ON community_follower
    FOR EACH ROW
    EXECUTE FUNCTION community_aggregates_subscriber_count ();

CREATE TRIGGER delete_follow_before_person
    BEFORE DELETE ON person
    FOR EACH ROW
    EXECUTE FUNCTION delete_follow_before_person ();

CREATE TRIGGER person_aggregates_comment_count
    AFTER INSERT OR DELETE OR UPDATE OF removed,
    deleted ON comment
    FOR EACH ROW
    EXECUTE FUNCTION person_aggregates_comment_count ();

CREATE TRIGGER person_aggregates_comment_score
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE FUNCTION person_aggregates_comment_score ();

CREATE TRIGGER person_aggregates_person
    AFTER INSERT OR DELETE ON person
    FOR EACH ROW
    EXECUTE FUNCTION person_aggregates_person ();

CREATE TRIGGER person_aggregates_post_count
    AFTER DELETE OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    EXECUTE FUNCTION person_aggregates_post_count ();

CREATE TRIGGER person_aggregates_post_insert
    AFTER INSERT ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION person_aggregates_post_insert ();

CREATE TRIGGER person_aggregates_post_score
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE FUNCTION person_aggregates_post_score ();

CREATE TRIGGER post_aggregates_comment_count
    AFTER INSERT OR DELETE OR UPDATE OF removed,
    deleted ON comment
    FOR EACH ROW
    EXECUTE FUNCTION post_aggregates_comment_count ();

CREATE TRIGGER post_aggregates_featured_community
    AFTER UPDATE ON post
    FOR EACH ROW
    WHEN ((old.featured_community IS DISTINCT FROM new.featured_community))
    EXECUTE FUNCTION post_aggregates_featured_community ();

CREATE TRIGGER post_aggregates_featured_local
    AFTER UPDATE ON post
    FOR EACH ROW
    WHEN ((old.featured_local IS DISTINCT FROM new.featured_local))
    EXECUTE FUNCTION post_aggregates_featured_local ();

CREATE TRIGGER post_aggregates_post
    AFTER INSERT ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION post_aggregates_post ();

CREATE TRIGGER post_aggregates_score
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE FUNCTION post_aggregates_score ();

CREATE TRIGGER site_aggregates_comment_delete
    AFTER DELETE OR UPDATE OF removed,
    deleted ON comment
    FOR EACH ROW
    WHEN ((old.local = TRUE))
    EXECUTE FUNCTION site_aggregates_comment_delete ();

CREATE TRIGGER site_aggregates_comment_insert
    AFTER INSERT OR UPDATE OF removed,
    deleted ON comment
    FOR EACH ROW
    WHEN ((new.local = TRUE))
    EXECUTE FUNCTION site_aggregates_comment_insert ();

CREATE TRIGGER site_aggregates_community_insert
    AFTER INSERT OR UPDATE OF removed,
    deleted ON community
    FOR EACH ROW
    WHEN ((new.local = TRUE))
    EXECUTE FUNCTION site_aggregates_community_insert ();

CREATE TRIGGER site_aggregates_person_delete
    AFTER DELETE ON person
    FOR EACH ROW
    WHEN ((old.local = TRUE))
    EXECUTE FUNCTION site_aggregates_person_delete ();

CREATE TRIGGER site_aggregates_person_insert
    AFTER INSERT ON person
    FOR EACH ROW
    WHEN ((new.local = TRUE))
    EXECUTE FUNCTION site_aggregates_person_insert ();

CREATE TRIGGER site_aggregates_post_delete
    AFTER DELETE OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    WHEN ((old.local = TRUE))
    EXECUTE FUNCTION site_aggregates_post_delete ();

CREATE TRIGGER site_aggregates_post_insert
    AFTER INSERT ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION site_aggregates_post_insert ();

CREATE TRIGGER site_aggregates_post_update
    AFTER UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    WHEN ((new.local = TRUE))
    EXECUTE FUNCTION site_aggregates_post_update ();

CREATE TRIGGER site_aggregates_site
    AFTER INSERT OR DELETE ON site
    FOR EACH ROW
    EXECUTE FUNCTION site_aggregates_site ();

-- Rank functions
CREATE FUNCTION controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS double precision
    LANGUAGE plpgsql
    IMMUTABLE
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
END;
$$;

CREATE FUNCTION hot_rank (score numeric, published timestamp with time zone)
    RETURNS double precision
    LANGUAGE plpgsql
    IMMUTABLE PARALLEL SAFE
    AS $$
DECLARE
    hours_diff numeric := EXTRACT(EPOCH FROM (now() - published)) / 3600;
BEGIN
    -- 24 * 7 = 168, so after a week, it will default to 0.
    IF (hours_diff > 0 AND hours_diff < 168) THEN
        -- Use greatest(2,score), so that the hot_rank will be positive and not ignored.
        RETURN log(greatest (2, score + 2)) / power((hours_diff + 2), 1.8);
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        RETURN 0.0;
    END IF;
END;
$$;

CREATE FUNCTION scaled_rank (score numeric, published timestamp with time zone, users_active_month numeric)
    RETURNS double precision
    LANGUAGE plpgsql
    IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    -- Add 2 to avoid divide by zero errors
    -- Default for score = 1, active users = 1, and now, is (0.1728 / log(2 + 1)) = 0.3621
    -- There may need to be a scale factor multiplied to users_active_month, to make
    -- the log curve less pronounced. This can be tuned in the future.
    RETURN (hot_rank (score, published) / log(2 + users_active_month));
END;
$$;

-- Don't defer constraints
ALTER TABLE comment_aggregates
    ALTER CONSTRAINT comment_aggregates_comment_id_fkey NOT DEFERRABLE;

ALTER TABLE community_aggregates
    ALTER CONSTRAINT community_aggregates_community_id_fkey NOT DEFERRABLE;

ALTER TABLE person_aggregates
    ALTER CONSTRAINT person_aggregates_person_id_fkey NOT DEFERRABLE;

ALTER TABLE post_aggregates
    ALTER CONSTRAINT post_aggregates_community_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT post_aggregates_creator_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT post_aggregates_instance_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT post_aggregates_post_id_fkey NOT DEFERRABLE;

ALTER TABLE site_aggregates
    ALTER CONSTRAINT site_aggregates_site_id_fkey NOT DEFERRABLE;

