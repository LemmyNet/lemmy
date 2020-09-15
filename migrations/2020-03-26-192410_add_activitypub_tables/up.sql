-- The Activitypub activity table
-- All user actions must create a row here.
create table activity (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null, -- Ensures that the user is set up here.
  data jsonb not null,
  local boolean not null default true,
  published timestamp not null default now(),
  updated timestamp
);

-- Making sure that id is unique
create unique index idx_activity_unique_apid on activity ((data ->> 'id'::text));

-- Add federation columns to the two actor tables
alter table user_ 
-- TODO uniqueness constraints should be added on these 3 columns later
add column actor_id character varying(255) not null default 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
add column bio text, -- not on community, already has description
add column local boolean not null default true,
add column private_key text, -- These need to be generated from code
add column public_key text,
add column last_refreshed_at timestamp not null default now() -- Used to re-fetch federated actor periodically
;

-- Community
alter table community 
add column actor_id character varying(255) not null default 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
add column local boolean not null default true,
add column private_key text, -- These need to be generated from code
add column public_key text,
add column last_refreshed_at timestamp not null default now() -- Used to re-fetch federated actor periodically
;

-- Don't worry about rebuilding the views right now.

