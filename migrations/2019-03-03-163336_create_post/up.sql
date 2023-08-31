CREATE TABLE post (
    id serial PRIMARY KEY,
    name varchar(100) NOT NULL,
    url text, -- These are both optional, a post can just have a title
    body text,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    removed boolean DEFAULT FALSE NOT NULL,
    locked boolean DEFAULT FALSE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

CREATE TABLE post_like (
    id serial PRIMARY KEY,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    score smallint NOT NULL, -- -1, or 1 for dislike, like, no row for no opinion
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (post_id, user_id)
);

CREATE TABLE post_saved (
    id serial PRIMARY KEY,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (post_id, user_id)
);

CREATE TABLE post_read (
    id serial PRIMARY KEY,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (post_id, user_id)
);

