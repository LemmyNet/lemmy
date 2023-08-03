CREATE TABLE comment (
    id serial PRIMARY KEY,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    parent_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    content text NOT NULL,
    removed boolean DEFAULT FALSE NOT NULL,
    read boolean DEFAULT FALSE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

CREATE TABLE comment_like (
    id serial PRIMARY KEY,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    score smallint NOT NULL, -- -1, or 1 for dislike, like, no row for no opinion
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (comment_id, user_id)
);

CREATE TABLE comment_saved (
    id serial PRIMARY KEY,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (comment_id, user_id)
);

