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

create or replace function site_aggregates_activity(i text) returns integer
    language plpgsql
    as $$
declare
   count_ integer;
begin
  select count(*)
  into count_
  from (
    select c.creator_id from comment c
    inner join person u on c.creator_id = u.id
    where c.published > ('now'::timestamp - i::interval) 
    and u.local = true
    union
    select p.creator_id from post p
    inner join person u on p.creator_id = u.id
    where p.published > ('now'::timestamp - i::interval)
    and u.local = true
  ) a;
  return count_;
end;
$$;
