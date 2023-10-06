CREATE VIEW mod_remove_post_view AS
SELECT
    mrp.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrp.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            post p
        WHERE
            mrp.post_id = p.id) AS post_name,
    (
        SELECT
            c.id
        FROM
            post p,
            community c
        WHERE
            mrp.post_id = p.id
            AND p.community_id = c.id) AS community_id,
    (
        SELECT
            c.name
        FROM
            post p,
            community c
        WHERE
            mrp.post_id = p.id
            AND p.community_id = c.id) AS community_name
FROM
    mod_remove_post mrp;

CREATE VIEW mod_lock_post_view AS
SELECT
    mlp.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mlp.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            post p
        WHERE
            mlp.post_id = p.id) AS post_name,
    (
        SELECT
            c.id
        FROM
            post p,
            community c
        WHERE
            mlp.post_id = p.id
            AND p.community_id = c.id) AS community_id,
    (
        SELECT
            c.name
        FROM
            post p,
            community c
        WHERE
            mlp.post_id = p.id
            AND p.community_id = c.id) AS community_name
FROM
    mod_lock_post mlp;

CREATE VIEW mod_remove_comment_view AS
SELECT
    mrc.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrc.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            c.id
        FROM
            comment c
        WHERE
            mrc.comment_id = c.id) AS comment_user_id,
    (
        SELECT
            name
        FROM
            user_ u,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND u.id = c.creator_id) AS comment_user_name,
    (
        SELECT
            content
        FROM
            comment c
        WHERE
            mrc.comment_id = c.id) AS comment_content,
    (
        SELECT
            p.id
        FROM
            post p,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id) AS post_id,
    (
        SELECT
            p.name
        FROM
            post p,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id) AS post_name,
    (
        SELECT
            co.id
        FROM
            comment c,
            post p,
            community co
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id
            AND p.community_id = co.id) AS community_id,
    (
        SELECT
            co.name
        FROM
            comment c,
            post p,
            community co
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id
            AND p.community_id = co.id) AS community_name
FROM
    mod_remove_comment mrc;

CREATE VIEW mod_remove_community_view AS
SELECT
    mrc.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrc.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            c.name
        FROM
            community c
        WHERE
            mrc.community_id = c.id) AS community_name
FROM
    mod_remove_community mrc;

CREATE VIEW mod_ban_from_community_view AS
SELECT
    mb.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.other_user_id = u.id) AS other_user_name,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            mb.community_id = c.id) AS community_name
FROM
    mod_ban_from_community mb;

CREATE VIEW mod_ban_view AS
SELECT
    mb.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.other_user_id = u.id) AS other_user_name
FROM
    mod_ban mb;

CREATE VIEW mod_add_community_view AS
SELECT
    ma.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.other_user_id = u.id) AS other_user_name,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            ma.community_id = c.id) AS community_name
FROM
    mod_add_community ma;

CREATE VIEW mod_add_view AS
SELECT
    ma.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.other_user_id = u.id) AS other_user_name
FROM
    mod_add ma;

