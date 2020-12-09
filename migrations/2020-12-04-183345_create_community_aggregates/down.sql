-- community aggregates
drop table community_aggregates;
drop trigger community_aggregates_community on community;
drop trigger community_aggregates_post_count on post;
drop trigger community_aggregates_comment_count on comment;
drop trigger community_aggregates_subscriber_count on community_follower;
drop function 
  community_aggregates_community,
  community_aggregates_post_count,
  community_aggregates_comment_count,
  community_aggregates_subscriber_count;
