-- Go through all the tables joins, optimize every view, CTE, etc.
create index idx_post_creator on post (creator_id);
create index idx_post_community on post (community_id);

create index idx_post_like_post on post_like (post_id);
create index idx_post_like_user on post_like (user_id);

create index idx_comment_creator on comment (creator_id);
create index idx_comment_parent on comment (parent_id);
create index idx_comment_post on comment (post_id);

create index idx_comment_like_comment on comment_like (comment_id);
create index idx_comment_like_user on comment_like (user_id);
create index idx_comment_like_post on comment_like (post_id);

create index idx_community_creator on community (creator_id);
create index idx_community_category on community (category_id);
