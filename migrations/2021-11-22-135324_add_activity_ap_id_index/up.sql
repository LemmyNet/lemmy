
-- Delete the empty ap_ids
delete from activity where ap_id is null;

-- Make it required
alter table activity alter column ap_id set not null;

-- Delete dupes, keeping the first one
delete from activity a using (
  select min(id) as id, ap_id
  from activity
  group by ap_id having count(*) > 1
) b
where a.ap_id = b.ap_id 
and a.id <> b.id;

-- The index
create unique index idx_activity_ap_id on activity(ap_id);

-- Drop the old index
drop index idx_activity_unique_apid;

