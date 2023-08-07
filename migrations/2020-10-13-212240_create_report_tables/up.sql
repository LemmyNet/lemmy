CREATE TABLE comment_report (
    id serial PRIMARY KEY,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- user reporting comment
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- comment being reported
    original_comment_text text NOT NULL,
    reason text NOT NULL,
    resolved bool NOT NULL DEFAULT FALSE,
    resolver_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE, -- user resolving report
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp NULL,
    UNIQUE (comment_id, creator_id) -- users should only be able to report a comment once
);

CREATE TABLE post_report (
    id serial PRIMARY KEY,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- user reporting post
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- post being reported
    original_post_name varchar(100) NOT NULL,
    original_post_url text,
    original_post_body text,
    reason text NOT NULL,
    resolved bool NOT NULL DEFAULT FALSE,
    resolver_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE, -- user resolving report
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp NULL,
    UNIQUE (post_id, creator_id) -- users should only be able to report a post once
);

CREATE OR REPLACE VIEW comment_report_view AS
SELECT
    cr.*,
    c.post_id,
    c.content AS current_comment_text,
    p.community_id,
    -- report creator details
    f.actor_id AS creator_actor_id,
    f.name AS creator_name,
    f.preferred_username AS creator_preferred_username,
    f.avatar AS creator_avatar,
    f.local AS creator_local,
    -- comment creator details
    u.id AS comment_creator_id,
    u.actor_id AS comment_creator_actor_id,
    u.name AS comment_creator_name,
    u.preferred_username AS comment_creator_preferred_username,
    u.avatar AS comment_creator_avatar,
    u.local AS comment_creator_local,
    -- resolver details
    r.actor_id AS resolver_actor_id,
    r.name AS resolver_name,
    r.preferred_username AS resolver_preferred_username,
    r.avatar AS resolver_avatar,
    r.local AS resolver_local
FROM
    comment_report cr
    LEFT JOIN comment c ON c.id = cr.comment_id
    LEFT JOIN post p ON p.id = c.post_id
    LEFT JOIN user_ u ON u.id = c.creator_id
    LEFT JOIN user_ f ON f.id = cr.creator_id
    LEFT JOIN user_ r ON r.id = cr.resolver_id;

CREATE OR REPLACE VIEW post_report_view AS
SELECT
    pr.*,
    p.name AS current_post_name,
    p.url AS current_post_url,
    p.body AS current_post_body,
    p.community_id,
    -- report creator details
    f.actor_id AS creator_actor_id,
    f.name AS creator_name,
    f.preferred_username AS creator_preferred_username,
    f.avatar AS creator_avatar,
    f.local AS creator_local,
    -- post creator details
    u.id AS post_creator_id,
    u.actor_id AS post_creator_actor_id,
    u.name AS post_creator_name,
    u.preferred_username AS post_creator_preferred_username,
    u.avatar AS post_creator_avatar,
    u.local AS post_creator_local,
    -- resolver details
    r.actor_id AS resolver_actor_id,
    r.name AS resolver_name,
    r.preferred_username AS resolver_preferred_username,
    r.avatar AS resolver_avatar,
    r.local AS resolver_local
FROM
    post_report pr
    LEFT JOIN post p ON p.id = pr.post_id
    LEFT JOIN user_ u ON u.id = p.creator_id
    LEFT JOIN user_ f ON f.id = pr.creator_id
    LEFT JOIN user_ r ON r.id = pr.resolver_id;

