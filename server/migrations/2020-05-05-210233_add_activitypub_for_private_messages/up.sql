alter table private_message
add column ap_id character varying(255) not null default 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
add column local boolean not null default true
;

drop materialized view private_message_mview;
drop view private_message_view;
create view private_message_view as 
select        
pm.*,
u.name as creator_name,
u.avatar as creator_avatar,
u.actor_id as creator_actor_id,
u.local as creator_local,
u2.name as recipient_name,
u2.avatar as recipient_avatar,
u2.actor_id as recipient_actor_id,
u2.local as recipient_local
from private_message pm
inner join user_ u on u.id = pm.creator_id
inner join user_ u2 on u2.id = pm.recipient_id;

create materialized view private_message_mview as select * from private_message_view;

create unique index idx_private_message_mview_id on private_message_mview (id);
