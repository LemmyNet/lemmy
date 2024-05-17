-- There is no restore for the contents of fast tables.
-- If you want to save past this point, you should make a DB backup.
CREATE TABLE comment_aggregates_fast (
    id integer NOT NULL,
    creator_id integer,
    post_id integer,
    parent_id integer,
    content text,
    removed boolean,
    read boolean,
    published timestamp without time zone,
    updated timestamp without time zone,
    deleted boolean,
    ap_id character varying(255),
    local boolean,
    post_name character varying(200),
    community_id integer,
    community_actor_id character varying(255),
    community_local boolean,
    community_name character varying(20),
    community_icon text,
    banned boolean,
    banned_from_community boolean,
    creator_actor_id character varying(255),
    creator_local boolean,
    creator_name character varying(20),
    creator_preferred_username character varying(20),
    creator_published timestamp without time zone,
    creator_avatar text,
    score bigint,
    upvotes bigint,
    downvotes bigint,
    hot_rank integer,
    hot_rank_active integer
);

CREATE TABLE community_aggregates_fast (
    id integer NOT NULL,
    name character varying(20),
    title character varying(100),
    icon text,
    banner text,
    description text,
    category_id integer,
    creator_id integer,
    removed boolean,
    published timestamp without time zone,
    updated timestamp without time zone,
    deleted boolean,
    nsfw boolean,
    actor_id character varying(255),
    local boolean,
    last_refreshed_at timestamp without time zone,
    creator_actor_id character varying(255),
    creator_local boolean,
    creator_name character varying(20),
    creator_preferred_username character varying(20),
    creator_avatar text,
    category_name character varying(100),
    number_of_subscribers bigint,
    number_of_posts bigint,
    number_of_comments bigint,
    hot_rank integer
);

CREATE TABLE post_aggregates_fast (
    id integer NOT NULL,
    name character varying(200),
    url text,
    body text,
    creator_id integer,
    community_id integer,
    removed boolean,
    locked boolean,
    published timestamp without time zone,
    updated timestamp without time zone,
    deleted boolean,
    nsfw boolean,
    stickied boolean,
    embed_title text,
    embed_description text,
    embed_html text,
    thumbnail_url text,
    ap_id character varying(255),
    local boolean,
    creator_actor_id character varying(255),
    creator_local boolean,
    creator_name character varying(20),
    creator_preferred_username character varying(20),
    creator_published timestamp without time zone,
    creator_avatar text,
    banned boolean,
    banned_from_community boolean,
    community_actor_id character varying(255),
    community_local boolean,
    community_name character varying(20),
    community_icon text,
    community_removed boolean,
    community_deleted boolean,
    community_nsfw boolean,
    number_of_comments bigint,
    score bigint,
    upvotes bigint,
    downvotes bigint,
    hot_rank integer,
    hot_rank_active integer,
    newest_activity_time timestamp without time zone
);

CREATE TABLE user_fast (
    id integer NOT NULL,
    actor_id character varying(255),
    name character varying(20),
    preferred_username character varying(20),
    avatar text,
    banner text,
    email text,
    matrix_user_id text,
    bio text,
    local boolean,
    admin boolean,
    banned boolean,
    show_avatars boolean,
    send_notifications_to_email boolean,
    published timestamp without time zone,
    number_of_posts bigint,
    post_score bigint,
    number_of_comments bigint,
    comment_score bigint
);

ALTER TABLE ONLY comment_aggregates_fast
    ADD CONSTRAINT comment_aggregates_fast_pkey PRIMARY KEY (id);

ALTER TABLE ONLY community_aggregates_fast
    ADD CONSTRAINT community_aggregates_fast_pkey PRIMARY KEY (id);

ALTER TABLE ONLY post_aggregates_fast
    ADD CONSTRAINT post_aggregates_fast_pkey PRIMARY KEY (id);

ALTER TABLE ONLY user_fast
    ADD CONSTRAINT user_fast_pkey PRIMARY KEY (id);

CREATE INDEX idx_post_aggregates_fast_hot_rank_active_published ON post_aggregates_fast USING btree (hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_fast_hot_rank_published ON post_aggregates_fast USING btree (hot_rank DESC, published DESC);

CREATE FUNCTION refresh_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        -- Update community number of comments
        UPDATE
            community_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments - 1
        FROM
            post AS p
        WHERE
            caf.id = p.community_id
            AND p.id = OLD.post_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id
        ON CONFLICT (id)
            DO NOTHING;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id;
        -- Update user view due to comment count
        UPDATE
            user_fast
        SET
            number_of_comments = number_of_comments + 1
        WHERE
            id = NEW.creator_id;
        -- Update post view due to comment count, new comment activity time, but only on new posts
        -- TODO this could be done more efficiently
        DELETE FROM post_aggregates_fast
        WHERE id = NEW.post_id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.post_id
        ON CONFLICT (id)
            DO NOTHING;
        -- Update the comment hot_ranks as of last week
        UPDATE
            comment_aggregates_fast AS caf
        SET
            hot_rank = cav.hot_rank,
            hot_rank_active = cav.hot_rank_active
        FROM
            comment_aggregates_view AS cav
        WHERE
            caf.id = cav.id
            AND (cav.published > ('now'::timestamp - '1 week'::interval));
        -- Update the post ranks
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = pav.hot_rank,
            hot_rank_active = pav.hot_rank_active
        FROM
            post_aggregates_view AS pav
        WHERE
            paf.id = pav.id
            AND (pav.published > ('now'::timestamp - '1 week'::interval));
        -- Force the hot rank active as zero on 2 day-older posts (necro-bump)
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank_active = 0
        WHERE
            paf.id = NEW.post_id
            AND (paf.published < ('now'::timestamp - '2 days'::interval));
        -- Update community number of comments
        UPDATE
            community_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments + 1
        FROM
            post AS p
        WHERE
            caf.id = p.community_id
            AND p.id = NEW.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_comment_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            comment_aggregates_fast
        SET
            score = CASE WHEN (OLD.score = 1) THEN
                score - 1
            ELSE
                score + 1
            END,
            upvotes = CASE WHEN (OLD.score = 1) THEN
                upvotes - 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (OLD.score = - 1) THEN
                downvotes - 1
            ELSE
                downvotes
            END
        WHERE
            id = OLD.comment_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            comment_aggregates_fast
        SET
            score = CASE WHEN (NEW.score = 1) THEN
                score + 1
            ELSE
                score - 1
            END,
            upvotes = CASE WHEN (NEW.score = 1) THEN
                upvotes + 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (NEW.score = - 1) THEN
                downvotes + 1
            ELSE
                downvotes
            END
        WHERE
            id = NEW.comment_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM community_aggregates_fast
        WHERE id = OLD.id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM community_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO community_aggregates_fast
        SELECT
            *
        FROM
            community_aggregates_view
        WHERE
            id = NEW.id
        ON CONFLICT (id)
            DO NOTHING;
        -- Update user view due to owner changes
        DELETE FROM user_fast
        WHERE id = NEW.creator_id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.creator_id
        ON CONFLICT (id)
            DO NOTHING;
        -- Update post view due to community changes
        DELETE FROM post_aggregates_fast
        WHERE community_id = NEW.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            community_id = NEW.id
        ON CONFLICT (id)
            DO NOTHING;
        -- TODO make sure this shows up in the users page ?
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO community_aggregates_fast
        SELECT
            *
        FROM
            community_aggregates_view
        WHERE
            id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_community_follower ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates_fast
        SET
            number_of_subscribers = number_of_subscribers - 1
        WHERE
            id = OLD.community_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates_fast
        SET
            number_of_subscribers = number_of_subscribers + 1
        WHERE
            id = NEW.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_community_user_ban ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            comment_aggregates_fast
        SET
            banned_from_community = FALSE
        WHERE
            creator_id = OLD.user_id
            AND community_id = OLD.community_id;
        UPDATE
            post_aggregates_fast
        SET
            banned_from_community = FALSE
        WHERE
            creator_id = OLD.user_id
            AND community_id = OLD.community_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            comment_aggregates_fast
        SET
            banned_from_community = TRUE
        WHERE
            creator_id = NEW.user_id
            AND community_id = NEW.community_id;
        UPDATE
            post_aggregates_fast
        SET
            banned_from_community = TRUE
        WHERE
            creator_id = NEW.user_id
            AND community_id = NEW.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        -- Update community number of posts
        UPDATE
            community_aggregates_fast
        SET
            number_of_posts = number_of_posts - 1
        WHERE
            id = OLD.community_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id
        ON CONFLICT (id)
            DO NOTHING;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id;
        -- Update that users number of posts, post score
        DELETE FROM user_fast
        WHERE id = NEW.creator_id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.creator_id
        ON CONFLICT (id)
            DO NOTHING;
        -- Update community number of posts
        UPDATE
            community_aggregates_fast
        SET
            number_of_posts = number_of_posts + 1
        WHERE
            id = NEW.community_id;
        -- Update the hot rank on the post table
        -- TODO this might not correctly update it, using a 1 week interval
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = pav.hot_rank,
            hot_rank_active = pav.hot_rank_active
        FROM
            post_aggregates_view AS pav
        WHERE
            paf.id = pav.id
            AND (pav.published > ('now'::timestamp - '1 week'::interval));
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_post_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            post_aggregates_fast
        SET
            score = CASE WHEN (OLD.score = 1) THEN
                score - 1
            ELSE
                score + 1
            END,
            upvotes = CASE WHEN (OLD.score = 1) THEN
                upvotes - 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (OLD.score = - 1) THEN
                downvotes - 1
            ELSE
                downvotes
            END
        WHERE
            id = OLD.post_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            post_aggregates_fast
        SET
            score = CASE WHEN (NEW.score = 1) THEN
                score + 1
            ELSE
                score - 1
            END,
            upvotes = CASE WHEN (NEW.score = 1) THEN
                upvotes + 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (NEW.score = - 1) THEN
                downvotes + 1
            ELSE
                downvotes
            END
        WHERE
            id = NEW.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_private_message ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY private_message_mview;
    RETURN NULL;
END
$$;

CREATE FUNCTION refresh_user ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM user_fast
        WHERE id = OLD.id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM user_fast
        WHERE id = OLD.id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.id
        ON CONFLICT (id)
            DO NOTHING;
        -- Refresh post_fast, cause of user info changes
        DELETE FROM post_aggregates_fast
        WHERE creator_id = NEW.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            creator_id = NEW.id
        ON CONFLICT (id)
            DO NOTHING;
        DELETE FROM comment_aggregates_fast
        WHERE creator_id = NEW.id;
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            creator_id = NEW.id
        ON CONFLICT (id)
            DO NOTHING;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_comment
    AFTER INSERT OR DELETE OR UPDATE ON comment
    FOR EACH ROW
    EXECUTE FUNCTION refresh_comment ();

CREATE TRIGGER refresh_comment_like
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE FUNCTION refresh_comment_like ();

CREATE TRIGGER refresh_community
    AFTER INSERT OR DELETE OR UPDATE ON community
    FOR EACH ROW
    EXECUTE FUNCTION refresh_community ();

CREATE TRIGGER refresh_community_follower
    AFTER INSERT OR DELETE ON community_follower
    FOR EACH ROW
    EXECUTE FUNCTION refresh_community_follower ();

CREATE TRIGGER refresh_community_user_ban
    AFTER INSERT OR DELETE ON community_user_ban
    FOR EACH ROW
    EXECUTE FUNCTION refresh_community_user_ban ();

CREATE TRIGGER refresh_post
    AFTER INSERT OR DELETE OR UPDATE ON post
    FOR EACH ROW
    EXECUTE FUNCTION refresh_post ();

CREATE TRIGGER refresh_post_like
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE FUNCTION refresh_post_like ();

CREATE TRIGGER refresh_user
    AFTER INSERT OR DELETE OR UPDATE ON user_
    FOR EACH ROW
    EXECUTE FUNCTION refresh_user ();

CREATE VIEW comment_aggregates_view AS
SELECT
    ct.id,
    ct.creator_id,
    ct.post_id,
    ct.parent_id,
    ct.content,
    ct.removed,
    ct.read,
    ct.published,
    ct.updated,
    ct.deleted,
    ct.ap_id,
    ct.local,
    p.name AS post_name,
    p.community_id,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name,
    c.icon AS community_icon,
    u.banned,
    (COALESCE(cb.id, 0))::boolean AS banned_from_community,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.published AS creator_published,
    u.avatar AS creator_avatar,
    COALESCE(cl.total, (0)::bigint) AS score,
    COALESCE(cl.up, (0)::bigint) AS upvotes,
    COALESCE(cl.down, (0)::bigint) AS downvotes,
    hot_rank ((COALESCE(cl.total, (1)::bigint))::numeric, p.published) AS hot_rank,
    hot_rank ((COALESCE(cl.total, (1)::bigint))::numeric, ct.published) AS hot_rank_active
FROM (((((comment ct
                LEFT JOIN post p ON (ct.post_id = p.id))
            LEFT JOIN community c ON (p.community_id = c.id))
        LEFT JOIN user_ u ON (ct.creator_id = u.id))
    LEFT JOIN community_user_ban cb ON (((ct.creator_id = cb.user_id)
                AND (p.id = ct.post_id)
                AND (p.community_id = cb.community_id))))
    LEFT JOIN (
        SELECT
            l.comment_id AS id,
            sum(l.score) AS total,
            count(
                CASE WHEN (l.score = 1) THEN
                    1
                ELSE
                    NULL::integer
                END) AS up,
            count(
                CASE WHEN (l.score = '-1'::integer) THEN
                    1
                ELSE
                    NULL::integer
                END) AS down
        FROM
            comment_like l
        GROUP BY
            l.comment_id) cl ON (cl.id = ct.id));

CREATE VIEW comment_fast_view AS
SELECT
    cav.id,
    cav.creator_id,
    cav.post_id,
    cav.parent_id,
    cav.content,
    cav.removed,
    cav.read,
    cav.published,
    cav.updated,
    cav.deleted,
    cav.ap_id,
    cav.local,
    cav.post_name,
    cav.community_id,
    cav.community_actor_id,
    cav.community_local,
    cav.community_name,
    cav.community_icon,
    cav.banned,
    cav.banned_from_community,
    cav.creator_actor_id,
    cav.creator_local,
    cav.creator_name,
    cav.creator_preferred_username,
    cav.creator_published,
    cav.creator_avatar,
    cav.score,
    cav.upvotes,
    cav.downvotes,
    cav.hot_rank,
    cav.hot_rank_active,
    us.user_id,
    us.my_vote,
    (us.is_subbed)::boolean AS subscribed,
    (us.is_saved)::boolean AS saved
FROM (comment_aggregates_fast cav
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user_id,
            COALESCE((cl.score)::integer, 0) AS my_vote,
            COALESCE(cf.id, 0) AS is_subbed,
            COALESCE(cs.id, 0) AS is_saved
        FROM (((user_ u
                    LEFT JOIN comment_like cl ON (((u.id = cl.user_id)
                                AND (cav.id = cl.comment_id))))
                LEFT JOIN comment_saved cs ON (((u.id = cs.user_id)
                            AND (cs.comment_id = cav.id))))
            LEFT JOIN community_follower cf ON (((u.id = cf.user_id)
                        AND (cav.community_id = cf.community_id))))) us)
UNION ALL
SELECT
    cav.id,
    cav.creator_id,
    cav.post_id,
    cav.parent_id,
    cav.content,
    cav.removed,
    cav.read,
    cav.published,
    cav.updated,
    cav.deleted,
    cav.ap_id,
    cav.local,
    cav.post_name,
    cav.community_id,
    cav.community_actor_id,
    cav.community_local,
    cav.community_name,
    cav.community_icon,
    cav.banned,
    cav.banned_from_community,
    cav.creator_actor_id,
    cav.creator_local,
    cav.creator_name,
    cav.creator_preferred_username,
    cav.creator_published,
    cav.creator_avatar,
    cav.score,
    cav.upvotes,
    cav.downvotes,
    cav.hot_rank,
    cav.hot_rank_active,
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS subscribed,
    NULL::boolean AS saved
FROM
    comment_aggregates_fast cav;

CREATE VIEW comment_report_view AS
SELECT
    cr.id,
    cr.creator_id,
    cr.comment_id,
    cr.original_comment_text,
    cr.reason,
    cr.resolved,
    cr.resolver_id,
    cr.published,
    cr.updated,
    c.post_id,
    c.content AS current_comment_text,
    p.community_id,
    f.actor_id AS creator_actor_id,
    f.name AS creator_name,
    f.preferred_username AS creator_preferred_username,
    f.avatar AS creator_avatar,
    f.local AS creator_local,
    u.id AS comment_creator_id,
    u.actor_id AS comment_creator_actor_id,
    u.name AS comment_creator_name,
    u.preferred_username AS comment_creator_preferred_username,
    u.avatar AS comment_creator_avatar,
    u.local AS comment_creator_local,
    r.actor_id AS resolver_actor_id,
    r.name AS resolver_name,
    r.preferred_username AS resolver_preferred_username,
    r.avatar AS resolver_avatar,
    r.local AS resolver_local
FROM (((((comment_report cr
                LEFT JOIN comment c ON (c.id = cr.comment_id))
            LEFT JOIN post p ON (p.id = c.post_id))
        LEFT JOIN user_ u ON (u.id = c.creator_id))
    LEFT JOIN user_ f ON (f.id = cr.creator_id))
    LEFT JOIN user_ r ON (r.id = cr.resolver_id));

CREATE VIEW comment_view AS
SELECT
    cav.id,
    cav.creator_id,
    cav.post_id,
    cav.parent_id,
    cav.content,
    cav.removed,
    cav.read,
    cav.published,
    cav.updated,
    cav.deleted,
    cav.ap_id,
    cav.local,
    cav.post_name,
    cav.community_id,
    cav.community_actor_id,
    cav.community_local,
    cav.community_name,
    cav.community_icon,
    cav.banned,
    cav.banned_from_community,
    cav.creator_actor_id,
    cav.creator_local,
    cav.creator_name,
    cav.creator_preferred_username,
    cav.creator_published,
    cav.creator_avatar,
    cav.score,
    cav.upvotes,
    cav.downvotes,
    cav.hot_rank,
    cav.hot_rank_active,
    us.user_id,
    us.my_vote,
    (us.is_subbed)::boolean AS subscribed,
    (us.is_saved)::boolean AS saved
FROM (comment_aggregates_view cav
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user_id,
            COALESCE((cl.score)::integer, 0) AS my_vote,
            COALESCE(cf.id, 0) AS is_subbed,
            COALESCE(cs.id, 0) AS is_saved
        FROM (((user_ u
                    LEFT JOIN comment_like cl ON (((u.id = cl.user_id)
                                AND (cav.id = cl.comment_id))))
                LEFT JOIN comment_saved cs ON (((u.id = cs.user_id)
                            AND (cs.comment_id = cav.id))))
            LEFT JOIN community_follower cf ON (((u.id = cf.user_id)
                        AND (cav.community_id = cf.community_id))))) us)
UNION ALL
SELECT
    cav.id,
    cav.creator_id,
    cav.post_id,
    cav.parent_id,
    cav.content,
    cav.removed,
    cav.read,
    cav.published,
    cav.updated,
    cav.deleted,
    cav.ap_id,
    cav.local,
    cav.post_name,
    cav.community_id,
    cav.community_actor_id,
    cav.community_local,
    cav.community_name,
    cav.community_icon,
    cav.banned,
    cav.banned_from_community,
    cav.creator_actor_id,
    cav.creator_local,
    cav.creator_name,
    cav.creator_preferred_username,
    cav.creator_published,
    cav.creator_avatar,
    cav.score,
    cav.upvotes,
    cav.downvotes,
    cav.hot_rank,
    cav.hot_rank_active,
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS subscribed,
    NULL::boolean AS saved
FROM
    comment_aggregates_view cav;

CREATE VIEW community_aggregates_view AS
SELECT
    c.id,
    c.name,
    c.title,
    c.icon,
    c.banner,
    c.description,
    c.category_id,
    c.creator_id,
    c.removed,
    c.published,
    c.updated,
    c.deleted,
    c.nsfw,
    c.actor_id,
    c.local,
    c.last_refreshed_at,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.avatar AS creator_avatar,
    cat.name AS category_name,
    COALESCE(cf.subs, (0)::bigint) AS number_of_subscribers,
    COALESCE(cd.posts, (0)::bigint) AS number_of_posts,
    COALESCE(cd.comments, (0)::bigint) AS number_of_comments,
    hot_rank ((cf.subs)::numeric, c.published) AS hot_rank
FROM ((((community c
            LEFT JOIN user_ u ON (c.creator_id = u.id))
        LEFT JOIN category cat ON (c.category_id = cat.id))
    LEFT JOIN (
        SELECT
            p.community_id,
            count(DISTINCT p.id) AS posts,
            count(DISTINCT ct.id) AS comments
        FROM (post p
            LEFT JOIN comment ct ON (p.id = ct.post_id))
    GROUP BY
        p.community_id) cd ON (cd.community_id = c.id))
    LEFT JOIN (
        SELECT
            community_follower.community_id,
            count(*) AS subs
        FROM
            community_follower
        GROUP BY
            community_follower.community_id) cf ON (cf.community_id = c.id));

CREATE VIEW community_fast_view AS
SELECT
    ac.id,
    ac.name,
    ac.title,
    ac.icon,
    ac.banner,
    ac.description,
    ac.category_id,
    ac.creator_id,
    ac.removed,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.nsfw,
    ac.actor_id,
    ac.local,
    ac.last_refreshed_at,
    ac.creator_actor_id,
    ac.creator_local,
    ac.creator_name,
    ac.creator_preferred_username,
    ac.creator_avatar,
    ac.category_name,
    ac.number_of_subscribers,
    ac.number_of_posts,
    ac.number_of_comments,
    ac.hot_rank,
    u.id AS user_id,
    (
        SELECT
            (cf.id)::boolean AS id
        FROM
            community_follower cf
        WHERE ((u.id = cf.user_id)
            AND (ac.id = cf.community_id))) AS subscribed
FROM (user_ u
    CROSS JOIN (
        SELECT
            ca.id,
            ca.name,
            ca.title,
            ca.icon,
            ca.banner,
            ca.description,
            ca.category_id,
            ca.creator_id,
            ca.removed,
            ca.published,
            ca.updated,
            ca.deleted,
            ca.nsfw,
            ca.actor_id,
            ca.local,
            ca.last_refreshed_at,
            ca.creator_actor_id,
            ca.creator_local,
            ca.creator_name,
            ca.creator_preferred_username,
            ca.creator_avatar,
            ca.category_name,
            ca.number_of_subscribers,
            ca.number_of_posts,
            ca.number_of_comments,
            ca.hot_rank
        FROM
            community_aggregates_fast ca) ac)
UNION ALL
SELECT
    caf.id,
    caf.name,
    caf.title,
    caf.icon,
    caf.banner,
    caf.description,
    caf.category_id,
    caf.creator_id,
    caf.removed,
    caf.published,
    caf.updated,
    caf.deleted,
    caf.nsfw,
    caf.actor_id,
    caf.local,
    caf.last_refreshed_at,
    caf.creator_actor_id,
    caf.creator_local,
    caf.creator_name,
    caf.creator_preferred_username,
    caf.creator_avatar,
    caf.category_name,
    caf.number_of_subscribers,
    caf.number_of_posts,
    caf.number_of_comments,
    caf.hot_rank,
    NULL::integer AS user_id,
    NULL::boolean AS subscribed
FROM
    community_aggregates_fast caf;

CREATE VIEW community_follower_view AS
SELECT
    cf.id,
    cf.community_id,
    cf.user_id,
    cf.published,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.preferred_username AS user_preferred_username,
    u.avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name,
    c.icon AS community_icon
FROM ((community_follower cf
    LEFT JOIN user_ u ON (cf.user_id = u.id))
    LEFT JOIN community c ON (cf.community_id = c.id));

CREATE VIEW community_moderator_view AS
SELECT
    cm.id,
    cm.community_id,
    cm.user_id,
    cm.published,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.preferred_username AS user_preferred_username,
    u.avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name,
    c.icon AS community_icon
FROM ((community_moderator cm
    LEFT JOIN user_ u ON (cm.user_id = u.id))
    LEFT JOIN community c ON (cm.community_id = c.id));

CREATE VIEW community_user_ban_view AS
SELECT
    cb.id,
    cb.community_id,
    cb.user_id,
    cb.published,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.preferred_username AS user_preferred_username,
    u.avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name,
    c.icon AS community_icon
FROM ((community_user_ban cb
    LEFT JOIN user_ u ON (cb.user_id = u.id))
    LEFT JOIN community c ON (cb.community_id = c.id));

CREATE VIEW community_view AS
SELECT
    cv.id,
    cv.name,
    cv.title,
    cv.icon,
    cv.banner,
    cv.description,
    cv.category_id,
    cv.creator_id,
    cv.removed,
    cv.published,
    cv.updated,
    cv.deleted,
    cv.nsfw,
    cv.actor_id,
    cv.local,
    cv.last_refreshed_at,
    cv.creator_actor_id,
    cv.creator_local,
    cv.creator_name,
    cv.creator_preferred_username,
    cv.creator_avatar,
    cv.category_name,
    cv.number_of_subscribers,
    cv.number_of_posts,
    cv.number_of_comments,
    cv.hot_rank,
    us."user" AS user_id,
    (us.is_subbed)::boolean AS subscribed
FROM (community_aggregates_view cv
    CROSS JOIN LATERAL (
        SELECT
            u.id AS "user",
            COALESCE(cf.community_id, 0) AS is_subbed
        FROM (user_ u
            LEFT JOIN community_follower cf ON (((u.id = cf.user_id)
                        AND (cf.community_id = cv.id))))) us)
UNION ALL
SELECT
    cv.id,
    cv.name,
    cv.title,
    cv.icon,
    cv.banner,
    cv.description,
    cv.category_id,
    cv.creator_id,
    cv.removed,
    cv.published,
    cv.updated,
    cv.deleted,
    cv.nsfw,
    cv.actor_id,
    cv.local,
    cv.last_refreshed_at,
    cv.creator_actor_id,
    cv.creator_local,
    cv.creator_name,
    cv.creator_preferred_username,
    cv.creator_avatar,
    cv.category_name,
    cv.number_of_subscribers,
    cv.number_of_posts,
    cv.number_of_comments,
    cv.hot_rank,
    NULL::integer AS user_id,
    NULL::boolean AS subscribed
FROM
    community_aggregates_view cv;

CREATE VIEW mod_add_community_view AS
SELECT
    id,
    mod_user_id,
    other_user_id,
    community_id,
    removed,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (ma.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        u.name
    FROM
        user_ u
    WHERE (ma.other_user_id = u.id)) AS other_user_name,
(
    SELECT
        c.name
    FROM
        community c
    WHERE (ma.community_id = c.id)) AS community_name
FROM
    mod_add_community ma;

CREATE VIEW mod_add_view AS
SELECT
    id,
    mod_user_id,
    other_user_id,
    removed,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (ma.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        u.name
    FROM
        user_ u
    WHERE (ma.other_user_id = u.id)) AS other_user_name
FROM
    mod_add ma;

CREATE VIEW mod_ban_from_community_view AS
SELECT
    id,
    mod_user_id,
    other_user_id,
    community_id,
    reason,
    banned,
    expires,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (mb.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        u.name
    FROM
        user_ u
    WHERE (mb.other_user_id = u.id)) AS other_user_name,
(
    SELECT
        c.name
    FROM
        community c
    WHERE (mb.community_id = c.id)) AS community_name
FROM
    mod_ban_from_community mb;

CREATE VIEW mod_ban_view AS
SELECT
    id,
    mod_user_id,
    other_user_id,
    reason,
    banned,
    expires,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (mb.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        u.name
    FROM
        user_ u
    WHERE (mb.other_user_id = u.id)) AS other_user_name
FROM
    mod_ban mb;

CREATE VIEW mod_lock_post_view AS
SELECT
    id,
    mod_user_id,
    post_id,
    LOCKED,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (mlp.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        p.name
    FROM
        post p
    WHERE (mlp.post_id = p.id)) AS post_name,
(
    SELECT
        c.id
    FROM
        post p,
        community c
    WHERE ((mlp.post_id = p.id)
        AND (p.community_id = c.id))) AS community_id,
(
    SELECT
        c.name
    FROM
        post p,
        community c
    WHERE ((mlp.post_id = p.id)
        AND (p.community_id = c.id))) AS community_name
FROM
    mod_lock_post mlp;

CREATE VIEW mod_remove_comment_view AS
SELECT
    id,
    mod_user_id,
    comment_id,
    reason,
    removed,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (mrc.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        c.id
    FROM
        comment c
    WHERE (mrc.comment_id = c.id)) AS comment_user_id,
(
    SELECT
        u.name
    FROM
        user_ u,
        comment c
    WHERE ((mrc.comment_id = c.id)
        AND (u.id = c.creator_id))) AS comment_user_name,
(
    SELECT
        c.content
    FROM
        comment c
    WHERE (mrc.comment_id = c.id)) AS comment_content,
(
    SELECT
        p.id
    FROM
        post p,
        comment c
    WHERE ((mrc.comment_id = c.id)
        AND (c.post_id = p.id))) AS post_id,
(
    SELECT
        p.name
    FROM
        post p,
        comment c
    WHERE ((mrc.comment_id = c.id)
        AND (c.post_id = p.id))) AS post_name,
(
    SELECT
        co.id
    FROM
        comment c,
        post p,
        community co
    WHERE ((mrc.comment_id = c.id)
        AND (c.post_id = p.id)
        AND (p.community_id = co.id))) AS community_id,
(
    SELECT
        co.name
    FROM
        comment c,
        post p,
        community co
    WHERE ((mrc.comment_id = c.id)
        AND (c.post_id = p.id)
        AND (p.community_id = co.id))) AS community_name
FROM
    mod_remove_comment mrc;

CREATE VIEW mod_remove_community_view AS
SELECT
    id,
    mod_user_id,
    community_id,
    reason,
    removed,
    expires,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (mrc.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        c.name
    FROM
        community c
    WHERE (mrc.community_id = c.id)) AS community_name
FROM
    mod_remove_community mrc;

CREATE VIEW mod_remove_post_view AS
SELECT
    id,
    mod_user_id,
    post_id,
    reason,
    removed,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (mrp.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        p.name
    FROM
        post p
    WHERE (mrp.post_id = p.id)) AS post_name,
(
    SELECT
        c.id
    FROM
        post p,
        community c
    WHERE ((mrp.post_id = p.id)
        AND (p.community_id = c.id))) AS community_id,
(
    SELECT
        c.name
    FROM
        post p,
        community c
    WHERE ((mrp.post_id = p.id)
        AND (p.community_id = c.id))) AS community_name
FROM
    mod_remove_post mrp;

CREATE VIEW mod_sticky_post_view AS
SELECT
    id,
    mod_user_id,
    post_id,
    stickied,
    when_,
    (
        SELECT
            u.name
        FROM
            user_ u
        WHERE (msp.mod_user_id = u.id)) AS mod_user_name,
(
    SELECT
        p.name
    FROM
        post p
    WHERE (msp.post_id = p.id)) AS post_name,
(
    SELECT
        c.id
    FROM
        post p,
        community c
    WHERE ((msp.post_id = p.id)
        AND (p.community_id = c.id))) AS community_id,
(
    SELECT
        c.name
    FROM
        post p,
        community c
    WHERE ((msp.post_id = p.id)
        AND (p.community_id = c.id))) AS community_name
FROM
    mod_sticky_post msp;

CREATE VIEW post_aggregates_view AS
SELECT
    p.id,
    p.name,
    p.url,
    p.body,
    p.creator_id,
    p.community_id,
    p.removed,
    p.locked,
    p.published,
    p.updated,
    p.deleted,
    p.nsfw,
    p.stickied,
    p.embed_title,
    p.embed_description,
    p.embed_html,
    p.thumbnail_url,
    p.ap_id,
    p.local,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.published AS creator_published,
    u.avatar AS creator_avatar,
    u.banned,
    (cb.id)::boolean AS banned_from_community,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name,
    c.icon AS community_icon,
    c.removed AS community_removed,
    c.deleted AS community_deleted,
    c.nsfw AS community_nsfw,
    COALESCE(ct.comments, (0)::bigint) AS number_of_comments,
    COALESCE(pl.score, (0)::bigint) AS score,
    COALESCE(pl.upvotes, (0)::bigint) AS upvotes,
    COALESCE(pl.downvotes, (0)::bigint) AS downvotes,
    hot_rank ((COALESCE(pl.score, (1)::bigint))::numeric, p.published) AS hot_rank,
    hot_rank ((COALESCE(pl.score, (1)::bigint))::numeric, GREATEST (ct.recent_comment_time, p.published)) AS hot_rank_active,
    GREATEST (ct.recent_comment_time, p.published) AS newest_activity_time
FROM (((((post p
                LEFT JOIN user_ u ON (p.creator_id = u.id))
            LEFT JOIN community_user_ban cb ON (((p.creator_id = cb.user_id)
                        AND (p.community_id = cb.community_id))))
        LEFT JOIN community c ON (p.community_id = c.id))
    LEFT JOIN (
        SELECT
            comment.post_id,
            count(*) AS comments,
            max(comment.published) AS recent_comment_time
        FROM
            comment
        GROUP BY
            comment.post_id) ct ON (ct.post_id = p.id))
    LEFT JOIN (
        SELECT
            post_like.post_id,
            sum(post_like.score) AS score,
            sum(post_like.score) FILTER (WHERE (post_like.score = 1)) AS upvotes,
        (- sum(post_like.score) FILTER (WHERE (post_like.score = '-1'::integer))) AS downvotes
FROM
    post_like
GROUP BY
    post_like.post_id) pl ON (pl.post_id = p.id))
ORDER BY
    p.id;

CREATE VIEW post_fast_view AS
SELECT
    pav.id,
    pav.name,
    pav.url,
    pav.body,
    pav.creator_id,
    pav.community_id,
    pav.removed,
    pav.locked,
    pav.published,
    pav.updated,
    pav.deleted,
    pav.nsfw,
    pav.stickied,
    pav.embed_title,
    pav.embed_description,
    pav.embed_html,
    pav.thumbnail_url,
    pav.ap_id,
    pav.local,
    pav.creator_actor_id,
    pav.creator_local,
    pav.creator_name,
    pav.creator_preferred_username,
    pav.creator_published,
    pav.creator_avatar,
    pav.banned,
    pav.banned_from_community,
    pav.community_actor_id,
    pav.community_local,
    pav.community_name,
    pav.community_icon,
    pav.community_removed,
    pav.community_deleted,
    pav.community_nsfw,
    pav.number_of_comments,
    pav.score,
    pav.upvotes,
    pav.downvotes,
    pav.hot_rank,
    pav.hot_rank_active,
    pav.newest_activity_time,
    us.id AS user_id,
    us.user_vote AS my_vote,
    (us.is_subbed)::boolean AS subscribed,
    (us.is_read)::boolean AS read,
    (us.is_saved)::boolean AS saved
FROM (post_aggregates_fast pav
    CROSS JOIN LATERAL (
        SELECT
            u.id,
            COALESCE(cf.community_id, 0) AS is_subbed,
            COALESCE(pr.post_id, 0) AS is_read,
            COALESCE(ps.post_id, 0) AS is_saved,
            COALESCE((pl.score)::integer, 0) AS user_vote
        FROM (((((user_ u
                            LEFT JOIN community_user_ban cb ON (((u.id = cb.user_id)
                                        AND (cb.community_id = pav.community_id))))
                        LEFT JOIN community_follower cf ON (((u.id = cf.user_id)
                                    AND (cf.community_id = pav.community_id))))
                    LEFT JOIN post_read pr ON (((u.id = pr.user_id)
                                AND (pr.post_id = pav.id))))
                LEFT JOIN post_saved ps ON (((u.id = ps.user_id)
                            AND (ps.post_id = pav.id))))
            LEFT JOIN post_like pl ON (((u.id = pl.user_id)
                        AND (pav.id = pl.post_id))))) us)
UNION ALL
SELECT
    pav.id,
    pav.name,
    pav.url,
    pav.body,
    pav.creator_id,
    pav.community_id,
    pav.removed,
    pav.locked,
    pav.published,
    pav.updated,
    pav.deleted,
    pav.nsfw,
    pav.stickied,
    pav.embed_title,
    pav.embed_description,
    pav.embed_html,
    pav.thumbnail_url,
    pav.ap_id,
    pav.local,
    pav.creator_actor_id,
    pav.creator_local,
    pav.creator_name,
    pav.creator_preferred_username,
    pav.creator_published,
    pav.creator_avatar,
    pav.banned,
    pav.banned_from_community,
    pav.community_actor_id,
    pav.community_local,
    pav.community_name,
    pav.community_icon,
    pav.community_removed,
    pav.community_deleted,
    pav.community_nsfw,
    pav.number_of_comments,
    pav.score,
    pav.upvotes,
    pav.downvotes,
    pav.hot_rank,
    pav.hot_rank_active,
    pav.newest_activity_time,
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS subscribed,
    NULL::boolean AS read,
    NULL::boolean AS saved
FROM
    post_aggregates_fast pav;

CREATE VIEW post_report_view AS
SELECT
    pr.id,
    pr.creator_id,
    pr.post_id,
    pr.original_post_name,
    pr.original_post_url,
    pr.original_post_body,
    pr.reason,
    pr.resolved,
    pr.resolver_id,
    pr.published,
    pr.updated,
    p.name AS current_post_name,
    p.url AS current_post_url,
    p.body AS current_post_body,
    p.community_id,
    f.actor_id AS creator_actor_id,
    f.name AS creator_name,
    f.preferred_username AS creator_preferred_username,
    f.avatar AS creator_avatar,
    f.local AS creator_local,
    u.id AS post_creator_id,
    u.actor_id AS post_creator_actor_id,
    u.name AS post_creator_name,
    u.preferred_username AS post_creator_preferred_username,
    u.avatar AS post_creator_avatar,
    u.local AS post_creator_local,
    r.actor_id AS resolver_actor_id,
    r.name AS resolver_name,
    r.preferred_username AS resolver_preferred_username,
    r.avatar AS resolver_avatar,
    r.local AS resolver_local
FROM ((((post_report pr
            LEFT JOIN post p ON (p.id = pr.post_id))
        LEFT JOIN user_ u ON (u.id = p.creator_id))
    LEFT JOIN user_ f ON (f.id = pr.creator_id))
    LEFT JOIN user_ r ON (r.id = pr.resolver_id));

CREATE VIEW post_view AS
SELECT
    pav.id,
    pav.name,
    pav.url,
    pav.body,
    pav.creator_id,
    pav.community_id,
    pav.removed,
    pav.locked,
    pav.published,
    pav.updated,
    pav.deleted,
    pav.nsfw,
    pav.stickied,
    pav.embed_title,
    pav.embed_description,
    pav.embed_html,
    pav.thumbnail_url,
    pav.ap_id,
    pav.local,
    pav.creator_actor_id,
    pav.creator_local,
    pav.creator_name,
    pav.creator_preferred_username,
    pav.creator_published,
    pav.creator_avatar,
    pav.banned,
    pav.banned_from_community,
    pav.community_actor_id,
    pav.community_local,
    pav.community_name,
    pav.community_icon,
    pav.community_removed,
    pav.community_deleted,
    pav.community_nsfw,
    pav.number_of_comments,
    pav.score,
    pav.upvotes,
    pav.downvotes,
    pav.hot_rank,
    pav.hot_rank_active,
    pav.newest_activity_time,
    us.id AS user_id,
    us.user_vote AS my_vote,
    (us.is_subbed)::boolean AS subscribed,
    (us.is_read)::boolean AS read,
    (us.is_saved)::boolean AS saved
FROM (post_aggregates_view pav
    CROSS JOIN LATERAL (
        SELECT
            u.id,
            COALESCE(cf.community_id, 0) AS is_subbed,
            COALESCE(pr.post_id, 0) AS is_read,
            COALESCE(ps.post_id, 0) AS is_saved,
            COALESCE((pl.score)::integer, 0) AS user_vote
        FROM (((((user_ u
                            LEFT JOIN community_user_ban cb ON (((u.id = cb.user_id)
                                        AND (cb.community_id = pav.community_id))))
                        LEFT JOIN community_follower cf ON (((u.id = cf.user_id)
                                    AND (cf.community_id = pav.community_id))))
                    LEFT JOIN post_read pr ON (((u.id = pr.user_id)
                                AND (pr.post_id = pav.id))))
                LEFT JOIN post_saved ps ON (((u.id = ps.user_id)
                            AND (ps.post_id = pav.id))))
            LEFT JOIN post_like pl ON (((u.id = pl.user_id)
                        AND (pav.id = pl.post_id))))) us)
UNION ALL
SELECT
    pav.id,
    pav.name,
    pav.url,
    pav.body,
    pav.creator_id,
    pav.community_id,
    pav.removed,
    pav.locked,
    pav.published,
    pav.updated,
    pav.deleted,
    pav.nsfw,
    pav.stickied,
    pav.embed_title,
    pav.embed_description,
    pav.embed_html,
    pav.thumbnail_url,
    pav.ap_id,
    pav.local,
    pav.creator_actor_id,
    pav.creator_local,
    pav.creator_name,
    pav.creator_preferred_username,
    pav.creator_published,
    pav.creator_avatar,
    pav.banned,
    pav.banned_from_community,
    pav.community_actor_id,
    pav.community_local,
    pav.community_name,
    pav.community_icon,
    pav.community_removed,
    pav.community_deleted,
    pav.community_nsfw,
    pav.number_of_comments,
    pav.score,
    pav.upvotes,
    pav.downvotes,
    pav.hot_rank,
    pav.hot_rank_active,
    pav.newest_activity_time,
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS subscribed,
    NULL::boolean AS read,
    NULL::boolean AS saved
FROM
    post_aggregates_view pav;

CREATE VIEW private_message_view AS
SELECT
    pm.id,
    pm.creator_id,
    pm.recipient_id,
    pm.content,
    pm.deleted,
    pm.read,
    pm.published,
    pm.updated,
    pm.ap_id,
    pm.local,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.avatar AS creator_avatar,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u2.name AS recipient_name,
    u2.preferred_username AS recipient_preferred_username,
    u2.avatar AS recipient_avatar,
    u2.actor_id AS recipient_actor_id,
    u2.local AS recipient_local
FROM ((private_message pm
        JOIN user_ u ON (u.id = pm.creator_id))
    JOIN user_ u2 ON (u2.id = pm.recipient_id));

CREATE VIEW reply_fast_view AS
WITH closereply AS (
    SELECT
        c2.id,
        c2.creator_id AS sender_id,
        c.creator_id AS recipient_id
    FROM (comment c
        JOIN comment c2 ON (c.id = c2.parent_id))
    WHERE (c2.creator_id <> c.creator_id)
UNION
SELECT
    c.id,
    c.creator_id AS sender_id,
    p.creator_id AS recipient_id
FROM
    comment c,
    post p
    WHERE ((c.post_id = p.id)
        AND (c.parent_id IS NULL)
        AND (c.creator_id <> p.creator_id)))
SELECT
    cv.id,
    cv.creator_id,
    cv.post_id,
    cv.parent_id,
    cv.content,
    cv.removed,
    cv.read,
    cv.published,
    cv.updated,
    cv.deleted,
    cv.ap_id,
    cv.local,
    cv.post_name,
    cv.community_id,
    cv.community_actor_id,
    cv.community_local,
    cv.community_name,
    cv.community_icon,
    cv.banned,
    cv.banned_from_community,
    cv.creator_actor_id,
    cv.creator_local,
    cv.creator_name,
    cv.creator_preferred_username,
    cv.creator_published,
    cv.creator_avatar,
    cv.score,
    cv.upvotes,
    cv.downvotes,
    cv.hot_rank,
    cv.hot_rank_active,
    cv.user_id,
    cv.my_vote,
    cv.subscribed,
    cv.saved,
    closereply.recipient_id
FROM
    comment_fast_view cv,
    closereply
WHERE (closereply.id = cv.id);

CREATE VIEW site_view AS
SELECT
    s.id,
    s.name,
    s.description,
    s.creator_id,
    s.published,
    s.updated,
    s.enable_downvotes,
    s.open_registration,
    s.enable_nsfw,
    s.icon,
    s.banner,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.avatar AS creator_avatar,
    (
        SELECT
            count(*) AS count
        FROM
            user_) AS number_of_users,
    (
        SELECT
            count(*) AS count
        FROM
            post) AS number_of_posts,
    (
        SELECT
            count(*) AS count
        FROM
            comment) AS number_of_comments,
    (
        SELECT
            count(*) AS count
        FROM
            community) AS number_of_communities
FROM (site s
    LEFT JOIN user_ u ON (s.creator_id = u.id));

CREATE VIEW user_mention_fast_view AS
SELECT
    ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.post_name,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.community_icon,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_preferred_username,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    ac.hot_rank_active,
    u.id AS user_id,
    COALESCE((cl.score)::integer, 0) AS my_vote,
    (
        SELECT
            (cs.id)::boolean AS id
        FROM
            comment_saved cs
        WHERE ((u.id = cs.user_id)
            AND (cs.comment_id = ac.id))) AS saved,
um.recipient_id,
(
    SELECT
        u_1.actor_id
    FROM
        user_ u_1
    WHERE (u_1.id = um.recipient_id)) AS recipient_actor_id,
(
    SELECT
        u_1.local
    FROM
        user_ u_1
    WHERE (u_1.id = um.recipient_id)) AS recipient_local
FROM (((user_ u
        CROSS JOIN (
            SELECT
                ca.id,
                ca.creator_id,
                ca.post_id,
                ca.parent_id,
                ca.content,
                ca.removed,
                ca.read,
                ca.published,
                ca.updated,
                ca.deleted,
                ca.ap_id,
                ca.local,
                ca.post_name,
                ca.community_id,
                ca.community_actor_id,
                ca.community_local,
                ca.community_name,
                ca.community_icon,
                ca.banned,
                ca.banned_from_community,
                ca.creator_actor_id,
                ca.creator_local,
                ca.creator_name,
                ca.creator_preferred_username,
                ca.creator_published,
                ca.creator_avatar,
                ca.score,
                ca.upvotes,
                ca.downvotes,
                ca.hot_rank,
                ca.hot_rank_active
            FROM
                comment_aggregates_fast ca) ac)
        LEFT JOIN comment_like cl ON (((u.id = cl.user_id)
                    AND (ac.id = cl.comment_id))))
    LEFT JOIN user_mention um ON (um.comment_id = ac.id))
UNION ALL
SELECT
    ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.post_name,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.community_icon,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_preferred_username,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    ac.hot_rank_active,
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS saved,
    um.recipient_id,
    (
        SELECT
            u.actor_id
        FROM
            user_ u
        WHERE (u.id = um.recipient_id)) AS recipient_actor_id,
(
    SELECT
        u.local
    FROM
        user_ u
    WHERE (u.id = um.recipient_id)) AS recipient_local
FROM (comment_aggregates_fast ac
    LEFT JOIN user_mention um ON (um.comment_id = ac.id));

CREATE VIEW user_mention_view AS
SELECT
    c.id,
    um.id AS user_mention_id,
    c.creator_id,
    c.creator_actor_id,
    c.creator_local,
    c.post_id,
    c.post_name,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.community_id,
    c.community_actor_id,
    c.community_local,
    c.community_name,
    c.community_icon,
    c.banned,
    c.banned_from_community,
    c.creator_name,
    c.creator_preferred_username,
    c.creator_avatar,
    c.score,
    c.upvotes,
    c.downvotes,
    c.hot_rank,
    c.hot_rank_active,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id,
    (
        SELECT
            u.actor_id
        FROM
            user_ u
        WHERE (u.id = um.recipient_id)) AS recipient_actor_id,
(
    SELECT
        u.local
    FROM
        user_ u
    WHERE (u.id = um.recipient_id)) AS recipient_local
FROM
    user_mention um,
    comment_view c
WHERE (um.comment_id = c.id);

CREATE VIEW user_view AS
SELECT
    u.id,
    u.actor_id,
    u.name,
    u.preferred_username,
    u.avatar,
    u.banner,
    u.email,
    u.matrix_user_id,
    u.bio,
    u.local,
    u.admin,
    u.banned,
    u.show_avatars,
    u.send_notifications_to_email,
    u.published,
    COALESCE(pd.posts, (0)::bigint) AS number_of_posts,
    COALESCE(pd.score, (0)::bigint) AS post_score,
    COALESCE(cd.comments, (0)::bigint) AS number_of_comments,
    COALESCE(cd.score, (0)::bigint) AS comment_score
FROM ((user_ u
    LEFT JOIN (
        SELECT
            p.creator_id,
            count(DISTINCT p.id) AS posts,
            sum(pl.score) AS score
        FROM (post p
            JOIN post_like pl ON (p.id = pl.post_id))
    GROUP BY
        p.creator_id) pd ON (u.id = pd.creator_id))
    LEFT JOIN (
        SELECT
            c.creator_id,
            count(DISTINCT c.id) AS comments,
            sum(cl.score) AS score
        FROM (comment c
            JOIN comment_like cl ON (c.id = cl.comment_id))
    GROUP BY
        c.creator_id) cd ON (u.id = cd.creator_id));

