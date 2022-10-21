-- Add back site columns
alter table site
  add column enable_downvotes boolean default true not null,
  add column open_registration boolean default true not null,
  add column enable_nsfw boolean default true not null,
  add column community_creation_admin_only boolean default false not null,
  add column require_email_verification boolean default false not null,
  add column require_application boolean default true not null,
  add column application_question text default 'to verify that you are human, please explain why you want to create an account on this site'::text,
  add column private_instance boolean default false not null,
  add column default_theme text default 'browser'::text not null,
  add column default_post_listing_type text default 'Local'::text not null,
  add column legal_information text,
  add column hide_modlog_mod_names boolean default true not null,
  add column application_email_admins boolean default false not null;

-- Insert the data back from local_site
update site set
  enable_downvotes = ls.enable_downvotes,
  open_registration = ls.open_registration,
  enable_nsfw = ls.enable_nsfw,
  community_creation_admin_only = ls.community_creation_admin_only,
  require_email_verification = ls.require_email_verification,
  require_application = ls.require_application,
  application_question = ls.application_question,
  private_instance = ls.private_instance,
  default_theme = ls.default_theme,
  default_post_listing_type = ls.default_post_listing_type,
  legal_information = ls.legal_information,
  hide_modlog_mod_names = ls.hide_modlog_mod_names,
  application_email_admins = ls.application_email_admins,
  published = ls.published,
  updated = ls.updated
from (select 
  site_id, 
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
from local_site) as ls
where site.id = ls.site_id;

-- drop instance columns
alter table site drop column instance_id;
alter table person drop column instance_id;
alter table community drop column instance_id;

drop table local_site_rate_limit;
drop table local_site;
drop table federation_allowlist;
drop table federation_blocklist;
drop table instance;
