-- Remove indexes
drop index idx_post_aggregates_controversy;
drop index idx_comment_aggregates_controversy;

-- Remove columns
alter table post_aggregates drop column controversy_rank;
alter table comment_aggregates drop column controversy_rank;

-- Remove function
drop function controversy_rank(numeric, numeric, numeric);

