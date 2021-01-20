-- Site aggregates
drop table site_aggregates;
drop trigger site_aggregates_site on site;
drop trigger site_aggregates_user_insert on user_;
drop trigger site_aggregates_user_delete on user_;
drop trigger site_aggregates_post_insert on post;
drop trigger site_aggregates_post_delete on post;
drop trigger site_aggregates_comment_insert on comment;
drop trigger site_aggregates_comment_delete on comment;
drop trigger site_aggregates_community_insert on community;
drop trigger site_aggregates_community_delete on community;
drop function 
  site_aggregates_site,
  site_aggregates_user_insert,
  site_aggregates_user_delete,
  site_aggregates_post_insert,
  site_aggregates_post_delete,
  site_aggregates_comment_insert,
  site_aggregates_comment_delete,
  site_aggregates_community_insert,
  site_aggregates_community_delete;
