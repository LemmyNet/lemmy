alter table site
    add column actor_id varchar(255) not null unique default generate_unique_changeme(),
    add column last_refreshed_at Timestamp not null default now(),
    add column inbox_url varchar(255) not null default generate_unique_changeme(),
    add column private_key text,
    add column public_key text not null default generate_unique_changeme();
