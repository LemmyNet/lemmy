-- Creating private message
CREATE TABLE private_message (
    id serial PRIMARY KEY,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    recipient_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    content text NOT NULL,
    deleted boolean DEFAULT FALSE NOT NULL,
    read boolean DEFAULT FALSE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

-- Create the view and materialized view which has the avatar and creator name
CREATE VIEW private_message_view AS
SELECT
    pm.*,
    u.name AS creator_name,
    u.avatar AS creator_avatar,
    u2.name AS recipient_name,
    u2.avatar AS recipient_avatar
FROM
    private_message pm
    INNER JOIN user_ u ON u.id = pm.creator_id
    INNER JOIN user_ u2 ON u2.id = pm.recipient_id;

CREATE MATERIALIZED VIEW private_message_mview AS
SELECT
    *
FROM
    private_message_view;

CREATE UNIQUE INDEX idx_private_message_mview_id ON private_message_mview (id);

-- Create the triggers
CREATE OR REPLACE FUNCTION refresh_private_message ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY private_message_mview;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_private_message
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON private_message
    FOR EACH statement
    EXECUTE PROCEDURE refresh_private_message ();

-- Update user to include matrix id
ALTER TABLE user_
    ADD COLUMN matrix_user_id text UNIQUE;

DROP VIEW user_view CASCADE;

CREATE VIEW user_view AS
SELECT
    u.id,
    u.name,
    u.avatar,
    u.email,
    u.matrix_user_id,
    u.fedi_name,
    u.admin,
    u.banned,
    u.show_avatars,
    u.send_notifications_to_email,
    u.published,
    (
        SELECT
            count(*)
        FROM
            post p
        WHERE
            p.creator_id = u.id) AS number_of_posts,
    (
        SELECT
            coalesce(sum(score), 0)
        FROM
            post p,
            post_like pl
        WHERE
            u.id = p.creator_id
            AND p.id = pl.post_id) AS post_score,
    (
        SELECT
            count(*)
        FROM
            comment c
        WHERE
            c.creator_id = u.id) AS number_of_comments,
    (
        SELECT
            coalesce(sum(score), 0)
        FROM
            comment c,
            comment_like cl
        WHERE
            u.id = c.creator_id
            AND c.id = cl.comment_id) AS comment_score
FROM
    user_ u;

CREATE MATERIALIZED VIEW user_mview AS
SELECT
    *
FROM
    user_view;

CREATE UNIQUE INDEX idx_user_mview_id ON user_mview (id);

-- This is what a group pm table would look like
-- Not going to do it now because of the complications
--
-- create table private_message (
--   id serial primary key,
--   creator_id int references user_ on update cascade on delete cascade not null,
--   content text not null,
--   deleted boolean default false not null,
--   published timestamp not null default now(),
--   updated timestamp
-- );
--
-- create table private_message_recipient (
--   id serial primary key,
--   private_message_id int references private_message on update cascade on delete cascade not null,
--   recipient_id int references user_ on update cascade on delete cascade not null,
--   read boolean default false not null,
--   published timestamp not null default now(),
--   unique(private_message_id, recipient_id)
-- )
