create table category (
  id serial primary key,
  name varchar(100) not null unique
);

insert into category (name) values
('Discussion'),
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

ALTER TABLE community ADD category_id int references category on update cascade on delete cascade not null default 1;