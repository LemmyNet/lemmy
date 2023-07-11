create table sent_activity (
    ap_id text primary key,
    data jsonb not null,
    sensitive boolean not null,
    published timestamp not null default now()
);

create table received_activity (
    ap_id text primary key,
    published timestamp not null default now()
);

insert into sent_activity(ap_id, data, sensitive, published)
    select ap_id, data, sensitive, published
    from activity
    where local = true;

insert into received_activity(ap_id, published)
    select ap_id, published
    from activity
    where local = false;

drop table activity;