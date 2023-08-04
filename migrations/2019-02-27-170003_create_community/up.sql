CREATE TABLE category (
    id serial PRIMARY KEY,
    name varchar(100) NOT NULL UNIQUE
);

INSERT INTO category (name)
    VALUES ('Discussion'),
    ('Humor/Memes'),
    ('Gaming'),
    ('Movies'),
    ('TV'),
    ('Music'),
    ('Literature'),
    ('Comics'),
    ('Photography'),
    ('Art'),
    ('Learning'),
    ('DIY'),
    ('Lifestyle'),
    ('News'),
    ('Politics'),
    ('Society'),
    ('Gender/Identity/Sexuality'),
    ('Race/Colonisation'),
    ('Religion'),
    ('Science/Technology'),
    ('Programming/Software'),
    ('Health/Sports/Fitness'),
    ('Porn'),
    ('Places'),
    ('Meta'),
    ('Other');

CREATE TABLE community (
    id serial PRIMARY KEY,
    name varchar(20) NOT NULL UNIQUE,
    title varchar(100) NOT NULL,
    description text,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    removed boolean DEFAULT FALSE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

CREATE TABLE community_moderator (
    id serial PRIMARY KEY,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (community_id, user_id)
);

CREATE TABLE community_follower (
    id serial PRIMARY KEY,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (community_id, user_id)
);

CREATE TABLE community_user_ban (
    id serial PRIMARY KEY,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (community_id, user_id)
);

INSERT INTO community (name, title, category_id, creator_id)
    VALUES ('main', 'The Default Community', 1, 1);

CREATE TABLE site (
    id serial PRIMARY KEY,
    name varchar(20) NOT NULL UNIQUE,
    description text,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

