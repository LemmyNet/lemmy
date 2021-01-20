-- post aggregates
drop table post_aggregates;
drop trigger post_aggregates_post on post;
drop trigger post_aggregates_comment_count on comment;
drop trigger post_aggregates_score on post_like;
drop trigger post_aggregates_stickied on post;
drop function 
  post_aggregates_post,
  post_aggregates_comment_count,
  post_aggregates_score,
  post_aggregates_stickied;
