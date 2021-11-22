
-- Delete the empty ap_ids
delete from activity where ap_id is null;

-- Make it required
alter table activity alter column ap_id set not null;

-- Delete dupes, keeping the first one
delete
from activity
where id not in (
  select min(id)
  from activity
  group by ap_id
);

-- The index
create unique index idx_activity_ap_id on activity(ap_id);

-- Drop the old index
drop index idx_activity_unique_apid;

