-- outgoing activities, need to be stored to be later server over http
-- we change data column from jsonb to json for decreased size
-- https://stackoverflow.com/a/22910602
create table sent_activity (
    id bigserial primary key,
    ap_id text unique not null,
    data json not null,
    sensitive boolean not null,
    published timestamp not null default now()
);

-- incoming activities, we only need the id to avoid processing the same activity multiple times
create table received_activity (
    id bigserial primary key,
    ap_id text unique not null,
    published timestamp not null default now()
);

-- copy sent activities to new table. only copy last 100k for faster migration
insert into sent_activity(ap_id, data, sensitive, published)
    select ap_id, data, sensitive, published
    from activity
    where local = true
    order by id desc
    limit 100000;

-- copy received activities to new table. only last 1m for faster migration
insert into received_activity(ap_id, published)
    select ap_id, published
    from activity
    where local = false
    order by id desc
    limit 1000000;

drop table activity;
