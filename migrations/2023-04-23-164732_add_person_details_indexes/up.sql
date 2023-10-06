-- Add a few indexes to speed up person details queries
CREATE INDEX idx_person_lower_name ON person (lower(name));

CREATE INDEX idx_community_lower_name ON community (lower(name));

CREATE INDEX idx_community_moderator_published ON community_moderator (published);

CREATE INDEX idx_community_moderator_community ON community_moderator (community_id);

CREATE INDEX idx_community_moderator_person ON community_moderator (person_id);

CREATE INDEX idx_comment_saved_comment ON comment_saved (comment_id);

CREATE INDEX idx_comment_saved_person ON comment_saved (person_id);

CREATE INDEX idx_community_block_community ON community_block (community_id);

CREATE INDEX idx_community_block_person ON community_block (person_id);

CREATE INDEX idx_community_follower_community ON community_follower (community_id);

CREATE INDEX idx_community_follower_person ON community_follower (person_id);

CREATE INDEX idx_person_block_person ON person_block (person_id);

CREATE INDEX idx_person_block_target ON person_block (target_id);

CREATE INDEX idx_post_language ON post (language_id);

CREATE INDEX idx_comment_language ON comment (language_id);

CREATE INDEX idx_person_aggregates_person ON person_aggregates (person_id);

CREATE INDEX idx_person_post_aggregates_post ON person_post_aggregates (post_id);

CREATE INDEX idx_person_post_aggregates_person ON person_post_aggregates (person_id);

CREATE INDEX idx_comment_reply_comment ON comment_reply (comment_id);

CREATE INDEX idx_comment_reply_recipient ON comment_reply (recipient_id);

CREATE INDEX idx_comment_reply_published ON comment_reply (published DESC);

