-- Add monthly and half yearly active columns for site and community aggregates

-- These columns don't need to be updated with a trigger, so they're saved daily via queries
alter table site_aggregates add column users_active_day bigint not null default 0;
alter table site_aggregates add column users_active_week bigint not null default 0;
alter table site_aggregates add column users_active_month bigint not null default 0;
alter table site_aggregates add column users_active_half_year bigint not null default 0;

alter table community_aggregates add column users_active_day bigint not null default 0;
alter table community_aggregates add column users_active_week bigint not null default 0;
alter table community_aggregates add column users_active_month bigint not null default 0;
alter table community_aggregates add column users_active_half_year bigint not null default 0;

create or replace function site_aggregates_activity(i text)
returns int
language plpgsql
as
$$
declare
   count_ integer;
begin
  select count(*) 
  into count_
  from (
    select c.creator_id from comment c
    inner join user_ u on c.creator_id = u.id
    where c.published > ('now'::timestamp - i::interval) 
    and u.local = true
    union
    select p.creator_id from post p
    inner join user_ u on p.creator_id = u.id
    where p.published > ('now'::timestamp - i::interval)
    and u.local = true
  ) a;
  return count_;
end;
$$;

update site_aggregates 
set users_active_day = (select * from site_aggregates_activity('1 day'));

update site_aggregates 
set users_active_week = (select * from site_aggregates_activity('1 week'));

update site_aggregates 
set users_active_month = (select * from site_aggregates_activity('1 month'));

update site_aggregates 
set users_active_half_year = (select * from site_aggregates_activity('6 months'));

create or replace function community_aggregates_activity(i text)
returns table(count_ bigint, community_id_ integer)
language plpgsql
as
$$
begin
  return query 
  select count(*), community_id
  from (
    select c.creator_id, p.community_id from comment c
    inner join post p on c.post_id = p.id
    where c.published > ('now'::timestamp - i::interval)
    union
    select p.creator_id, p.community_id from post p
    where p.published > ('now'::timestamp - i::interval)  
  ) a
  group by community_id;
end;
$$;

update community_aggregates ca
set users_active_day = mv.count_
from community_aggregates_activity('1 day') mv
where ca.community_id = mv.community_id_;

update community_aggregates ca
set users_active_week = mv.count_
from community_aggregates_activity('1 week') mv
where ca.community_id = mv.community_id_;

update community_aggregates ca
set users_active_month = mv.count_
from community_aggregates_activity('1 month') mv
where ca.community_id = mv.community_id_;

update community_aggregates ca
set users_active_half_year = mv.count_
from community_aggregates_activity('6 months') mv
where ca.community_id = mv.community_id_;
