alter table local_site add column content_warning text;
alter table community add column only_followers_can_vote boolean not null default false;