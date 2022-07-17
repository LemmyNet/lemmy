
-- Remove the comment.read column, and create a new comment_reply table,
-- similar to the person_mention table. 
-- 
-- This is necessary because self-joins using ltrees would be too tough with SQL views
-- 
-- Every comment should have a row here, because all comments have a recipient, 
-- either the post creator, or the parent commenter.
create table comment_reply(
  id serial primary key,
  recipient_id int references person on update cascade on delete cascade not null,
  comment_id int references comment on update cascade on delete cascade not null,
  read boolean default false not null,
  published timestamp not null default now(),
  unique(recipient_id, comment_id)
);

-- Ones where parent_id is null, use the post creator recipient
insert into comment_reply (recipient_id, comment_id, read)
select p.creator_id, c.id, c.read from comment c
inner join post p on c.post_id = p.id
where c.parent_id is null;

--  Ones where there is a parent_id, self join to comment to get the parent comment creator
insert into comment_reply (recipient_id, comment_id, read)
select c2.creator_id, c.id, c.read from comment c
inner join comment c2 on c.parent_id = c2.id;

-- Drop comment_alias view
drop view comment_alias_1;

alter table comment drop column read;

create extension ltree;

alter table comment add column path ltree not null default '0';
alter table comment_aggregates add column child_count integer not null default 0;

-- The ltree path column should be the comment_id parent paths, separated by dots. 
-- Stackoverflow: building an ltree from a parent_id hierarchical tree:
-- https://stackoverflow.com/a/1144848/1655478

create temporary table comment_temp as 
WITH RECURSIVE q AS (
	SELECT  h, 1 AS level, ARRAY[id] AS breadcrumb
	FROM    comment h
	WHERE   parent_id is null
	UNION ALL
	SELECT  hi, q.level + 1 AS level, breadcrumb || id
	FROM    q
	JOIN    comment hi
	ON      hi.parent_id = (q.h).id
)
SELECT  (q.h).id,
	(q.h).parent_id,
	level,
	breadcrumb::VARCHAR AS path,
	text2ltree('0.' || array_to_string(breadcrumb, '.')) as ltree_path
FROM    q
ORDER BY
	breadcrumb;

-- Add the ltree column
update comment c 
set path = ct.ltree_path
from comment_temp ct
where c.id = ct.id;

-- Update the child counts
update comment_aggregates ca set child_count = c2.child_count
from (
  select c.id, c.path, count(c2.id) as child_count from comment c
  left join comment c2 on c2.path <@ c.path and c2.path != c.path
  group by c.id
) as c2
where ca.comment_id = c2.id;

-- Create the index
create index idx_path_gist on comment using gist (path);

-- Drop the parent_id column
alter table comment drop column parent_id cascade;

