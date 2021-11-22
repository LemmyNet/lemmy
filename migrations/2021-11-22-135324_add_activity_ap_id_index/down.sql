alter table activity alter column ap_id drop not null;

create unique index idx_activity_unique_apid on activity ((data ->> 'id'::text));

drop index idx_activity_ap_id;
