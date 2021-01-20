-- User aggregates
drop table user_aggregates;
drop trigger user_aggregates_user on user_;
drop trigger user_aggregates_post_count on post;
drop trigger user_aggregates_post_score on post_like;
drop trigger user_aggregates_comment_count on comment;
drop trigger user_aggregates_comment_score on comment_like;
drop function 
  user_aggregates_user, 
  user_aggregates_post_count,
  user_aggregates_post_score,
  user_aggregates_comment_count,
  user_aggregates_comment_score;
