CREATE TABLE mod_remove_post (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    removed boolean DEFAULT TRUE,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE mod_lock_post (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    locked boolean DEFAULT TRUE,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE mod_remove_comment (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    removed boolean DEFAULT TRUE,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE mod_remove_community (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    removed boolean DEFAULT TRUE,
    expires timestamp,
    when_ timestamp NOT NULL DEFAULT now()
);

-- TODO make sure you can't ban other mods
CREATE TABLE mod_ban_from_community (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    other_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    banned boolean DEFAULT TRUE,
    expires timestamp,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE mod_ban (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    other_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    banned boolean DEFAULT TRUE,
    expires timestamp,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE mod_add_community (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    other_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    removed boolean DEFAULT FALSE,
    when_ timestamp NOT NULL DEFAULT now()
);

-- When removed is false that means kicked
CREATE TABLE mod_add (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    other_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    removed boolean DEFAULT FALSE,
    when_ timestamp NOT NULL DEFAULT now()
);

