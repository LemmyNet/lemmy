-- Create an instance table
-- Holds any connected or unconnected domain
create table instance (
  id serial primary key,
  domain varchar(255) not null unique,
  published timestamp not null default now(),
  updated timestamp null
);

-- Insert all the domains to the instance table
insert into instance (domain)
select distinct substring(p.actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') from ( 
  select actor_id from site 
  union 
  select actor_id from person 
  union 
  select actor_id from community
) as p;

-- Alter site, person, and community tables to reference the instance table.
alter table site add column 
instance_id int references instance on update cascade on delete cascade;

alter table person add column 
instance_id int references instance on update cascade on delete cascade;

alter table community add column 
instance_id int references instance on update cascade on delete cascade;

-- Add those columns
update site set instance_id = i.id 
from instance i
where substring(actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

update person set instance_id = i.id 
from instance i
where substring(actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

update community set instance_id = i.id 
from instance i
where substring(actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

-- Make those columns unique not null now
alter table site alter column instance_id set not null;
alter table site add constraint idx_site_instance_unique unique (instance_id);

alter table person alter column instance_id set not null;
alter table community alter column instance_id set not null;

-- Create allowlist and blocklist tables
create table federation_allowlist (
  id serial primary key,
  instance_id int references instance on update cascade on delete cascade not null unique,
  published timestamp not null default now(),
  updated timestamp null
);

create table federation_blocklist (
  id serial primary key,
  instance_id int references instance on update cascade on delete cascade not null unique,
  published timestamp not null default now(),
  updated timestamp null
);

-- Move all the extra site settings-type columns to a local_site table
-- Add a lot of other fields currently in the lemmy.hjson
create table local_site (
  id serial primary key,
  site_id int references site on update cascade on delete cascade not null unique,

  -- Site table fields
  site_setup boolean default false not null,
  enable_downvotes boolean default true not null,
  open_registration boolean default true not null,
  enable_nsfw boolean default true not null,
  community_creation_admin_only boolean default false not null,
  require_email_verification boolean default false not null,
  require_application boolean default true not null,
  application_question text default 'to verify that you are human, please explain why you want to create an account on this site'::text,
  private_instance boolean default false not null,
  default_theme text default 'browser'::text not null,
  default_post_listing_type text default 'Local'::text not null,
  legal_information text,
  hide_modlog_mod_names boolean default true not null,
  application_email_admins boolean default false not null,

  -- Fields from lemmy.hjson
  slur_filter_regex text,
  actor_name_max_length int default 20 not null,
  federation_enabled boolean default true not null,
  federation_debug boolean default false not null,
  federation_strict_allowlist boolean default true not null,
  federation_http_fetch_retry_limit int default 25 not null,
  federation_worker_count int default 64 not null,
  captcha_enabled boolean default false not null,
  captcha_difficulty varchar(255) default 'medium' not null,

  -- Time fields
  published timestamp without time zone default now() not null,
  updated timestamp without time zone
);

-- local_site_rate_limit is its own table, so as to not go over 32 columns, and force diesel to use the 64-column-tables feature
create table local_site_rate_limit (
  id serial primary key,
  local_site_id int references local_site on update cascade on delete cascade not null unique,
  message int default 180 not null,
  message_per_second int default 60 not null,
  post int default 6 not null,
  post_per_second int default 600 not null,
  register int default 3 not null,
  register_per_second int default 3600 not null,
  image int default 6 not null,
  image_per_second int default 3600 not null,
  comment int default 6 not null,
  comment_per_second int default 600 not null,
  search int default 60 not null,
  search_per_second int default 600 not null,
  published timestamp without time zone default now() not null,
  updated timestamp without time zone
);

-- Insert the data into local_site
insert into local_site (
  site_id, 
  site_setup,
  enable_downvotes,
  open_registration,
  enable_nsfw,
  community_creation_admin_only,
  require_email_verification,
  require_application,
  application_question,
  private_instance,
  default_theme,
  default_post_listing_type,
  legal_information,
  hide_modlog_mod_names,
  application_email_admins,
  published,
  updated
) 
select 
  id, 
  true, -- Assume site if setup if there's already a site row
  enable_downvotes,
  open_registration,
  enable_nsfw,
  community_creation_admin_only,
  require_email_verification,
  require_application,
  application_question,
  private_instance,
  default_theme,
  default_post_listing_type,
  legal_information,
  hide_modlog_mod_names,
  application_email_admins,
  published,
  updated
from site
order by id limit 1;

-- Default here
insert into local_site_rate_limit (
  local_site_id
)
select id from local_site
order by id limit 1;

-- Drop all those columns from site
alter table site
  drop column enable_downvotes,
  drop column open_registration,
  drop column enable_nsfw,
  drop column community_creation_admin_only,
  drop column require_email_verification,
  drop column require_application,
  drop column application_question,
  drop column private_instance,
  drop column default_theme,
  drop column default_post_listing_type,
  drop column legal_information,
  drop column hide_modlog_mod_names,
  drop column application_email_admins;

