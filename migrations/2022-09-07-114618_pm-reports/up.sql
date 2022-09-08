create table private_message_report (
  id            serial    primary key,
  creator_id    int       references person on update cascade on delete cascade not null,   -- user reporting comment
  private_message_id    int       references private_message on update cascade on delete cascade not null, -- comment being reported
  original_pm_text  text      not null,
  reason        text      not null,
  resolved      bool      not null default false,
  resolver_id   int       references person on update cascade on delete cascade,   -- user resolving report
  published     timestamp not null default now(),
  updated       timestamp null,
  unique(private_message_id, creator_id) -- users should only be able to report a pm once
);

create or replace view private_message_report_view as
select pmr.*,
c.content as current_pm_text,
-- report creator details
f.actor_id as creator_actor_id,
f.name as creator_name,
f.avatar as creator_avatar,
f.local as creator_local,
-- pm creator details
u.id as pm_creator_id,
u.actor_id as pm_creator_actor_id,
u.name as pm_creator_name,
u.avatar as pm_creator_avatar,
u.local as pm_creator_local,
-- resolver details
r.actor_id as resolver_actor_id,
r.name as resolver_name,
r.avatar as resolver_avatar,
r.local as resolver_local
from private_message_report pmr
left join private_message c on c.id = pmr.private_message_id
left join person u on u.id = c.creator_id
left join person f on f.id = pmr.creator_id
left join person r on r.id = pmr.resolver_id;