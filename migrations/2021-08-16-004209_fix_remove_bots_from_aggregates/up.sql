-- Make sure bots aren't included in aggregate counts

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
    inner join person pe on c.creator_id = pe.id
    where c.published > ('now'::timestamp - i::interval)
    and pe.bot_account = false
    union
    select p.creator_id, p.community_id from post p
    inner join person pe on p.creator_id = pe.id
    where p.published > ('now'::timestamp - i::interval)  
    and pe.bot_account = false
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
    inner join person pe on c.creator_id = pe.id
    where c.published > ('now'::timestamp - i::interval) 
    and u.local = true
    and pe.bot_account = false
    union
    select p.creator_id from post p
    inner join person u on p.creator_id = u.id
    inner join person pe on p.creator_id = pe.id
    where p.published > ('now'::timestamp - i::interval)
    and u.local = true
    and pe.bot_account = false
  ) a;
  return count_;
end;
$$;
