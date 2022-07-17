alter table comment add column parent_id integer;

-- Constraints and index
alter table comment add constraint comment_parent_id_fkey foreign key (parent_id) REFERENCES comment(id) ON UPDATE CASCADE ON DELETE CASCADE;
create index idx_comment_parent on comment (parent_id);

-- Update the parent_id column
-- subpath(subpath(0, -1), -1) gets the immediate parent but it fails null checks
update comment set parent_id = cast(ltree2text(nullif(subpath(nullif(subpath(path, 0, -1), '0'), -1), '0')) as INTEGER);

alter table comment drop column path;
alter table comment_aggregates drop column child_count;

drop extension ltree;

-- Add back in the read column
alter table comment add column read boolean default false not null;

update comment c set read = cr.read
from comment_reply cr where cr.comment_id = c.id;

create view comment_alias_1 as select * from comment;    

drop table comment_reply;

