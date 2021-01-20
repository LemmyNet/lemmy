-- comment aggregates
drop table comment_aggregates;
drop trigger comment_aggregates_comment on comment;
drop trigger comment_aggregates_score on comment_like;
drop function 
  comment_aggregates_comment,
  comment_aggregates_score;
