create table activity (
    id serial primary key,
    data jsonb not null,
    local boolean not null default true,
    published timestamp not null default now(),
    updated timestamp,
    ap_id text not null,
    sensitive boolean not null default true
);

insert into activity(ap_id, data, sensitive, published)
    select ap_id, data, sensitive, published
    from sent_activity
    order by id desc
    limit 100000;

-- We cant copy received_activity entries back into activities table because we dont have data
-- which is mandatory.

drop table sent_activity;
drop table received_activity;