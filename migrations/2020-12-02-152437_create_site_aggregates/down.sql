-- Site aggregates
drop table site_aggregates;
drop trigger site_aggregates_user on user_;
drop trigger site_aggregates_post on post;
drop trigger site_aggregates_comment on comment;
drop trigger site_aggregates_community on community;
drop function 
  site_aggregates_user,
  site_aggregates_post,
  site_aggregates_comment,
  site_aggregates_community;
