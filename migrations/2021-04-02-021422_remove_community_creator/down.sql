
--  Add the column back
alter table community add column creator_id int references person on update cascade on delete cascade;

-- Recreate the index
create index idx_community_creator on community (creator_id);

-- Add the data, selecting the highest mod
update community
set creator_id = sub.person_id
from (
  select 
  cm.community_id,
  cm.person_id
  from 
  community_moderator cm
  limit 1
) as sub
where id = sub.community_id;

-- Set to not null
alter table community alter column creator_id set not null;


