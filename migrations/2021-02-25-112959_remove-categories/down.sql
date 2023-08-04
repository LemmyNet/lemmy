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

ALTER TABLE community
    ADD category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL DEFAULT 1;

