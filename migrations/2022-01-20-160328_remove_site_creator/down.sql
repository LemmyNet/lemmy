--  Add the column back
alter table site add column creator_id int references person on update cascade on delete cascade;

-- Add the data, selecting the highest admin
update site
set creator_id = sub.id
from (
  select id from person
  where admin = true
  limit 1
) as sub;

-- Set to not null
alter table site alter column creator_id set not null;
