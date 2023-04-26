-- Add a few indexes to speed up person details queries
create index idx_person_lower_name on person (lower(name));
create index idx_community_lower_name on community (lower(name));

create index idx_community_moderator_published on community_moderator (published);
create index idx_community_moderator_community on community_moderator (community_id);
create index idx_community_moderator_person on community_moderator (person_id);

create index idx_comment_saved_comment on comment_saved (comment_id);
create index idx_comment_saved_person on comment_saved (person_id);

create index idx_community_block_community on community_block (community_id);
create index idx_community_block_person on community_block (person_id);

create index idx_community_follower_community on community_follower (community_id);
create index idx_community_follower_person on community_follower (person_id);

create index idx_person_block_person on person_block (person_id);
create index idx_person_block_target on person_block (target_id);

create index idx_post_language on post (language_id);
create index idx_comment_language on comment (language_id);

create index idx_person_aggregates_person on person_aggregates (person_id);

create index idx_person_post_aggregates_post on person_post_aggregates (post_id);
create index idx_person_post_aggregates_person on person_post_aggregates (person_id);

create index idx_comment_reply_comment on comment_reply (comment_id);
create index idx_comment_reply_recipient on comment_reply (recipient_id);
create index idx_comment_reply_published on comment_reply (published desc);
