-- Drop triggers
drop trigger if exists refresh_comment on comment;
drop trigger if exists refresh_comment_like on comment_like;
drop trigger if exists refresh_community on community;
drop trigger if exists refresh_community_follower on community_follower;
drop trigger if exists refresh_community_user_ban on community_user_ban;
drop trigger if exists refresh_post on post;
drop trigger if exists refresh_post_like on post_like;
drop trigger if exists refresh_user on user_;

-- Drop functions
drop function if exists
refresh_comment,
refresh_comment_like,
refresh_community,
refresh_community_follower,
refresh_community_user_ban,
refresh_post,
refresh_post_like,
refresh_private_message,
refresh_user
cascade;

-- Drop views
drop view if exists
comment_aggregates_view, 
comment_fast_view,
comment_report_view,
comment_view,
community_aggregates_view,
community_fast_view,
community_follower_view,
community_moderator_view,
community_user_ban_view,
community_view,
mod_add_community_view,
mod_add_view,
mod_ban_from_community_view,
mod_ban_view,
mod_lock_post_view,
mod_remove_comment_view,
mod_remove_community_view,
mod_remove_post_view,
mod_sticky_post_view,
post_aggregates_view,
post_fast_view,
post_report_view,
post_view,
private_message_view,
reply_fast_view,
site_view,
user_mention_fast_view,
user_mention_view,
user_view
cascade;

-- Drop fast tables
drop table if exists
comment_aggregates_fast,
community_aggregates_fast,
post_aggregates_fast,
user_fast
cascade;

