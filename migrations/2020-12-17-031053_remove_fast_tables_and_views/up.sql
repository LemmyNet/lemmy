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

