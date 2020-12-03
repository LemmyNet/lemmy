-- Site aggregates
drop table site_aggregates;
drop trigger site_aggregates_insert_user on user_;
drop trigger site_aggregates_delete_user on user_;
drop trigger site_aggregates_insert_post on post;
drop trigger site_aggregates_delete_post on post;
drop trigger site_aggregates_insert_comment on comment;
drop trigger site_aggregates_delete_comment on comment;
drop trigger site_aggregates_insert_community on community;
drop trigger site_aggregates_delete_community on community;
drop function 
  site_aggregates_user_increment,
  site_aggregates_user_decrement,
  site_aggregates_post_increment,
  site_aggregates_post_decrement,
  site_aggregates_comment_increment,
  site_aggregates_comment_decrement,
  site_aggregates_community_increment,
  site_aggregates_community_decrement;
