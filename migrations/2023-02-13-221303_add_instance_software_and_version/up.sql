-- Add Software and Version columns from nodeinfo to the instance table

alter table instance add column software varchar(255);
alter table instance add column version varchar(255);
