-- community aggregates
DROP TABLE community_aggregates;

DROP TRIGGER community_aggregates_community ON community;

DROP TRIGGER community_aggregates_post_count ON post;

DROP TRIGGER community_aggregates_comment_count ON comment;

DROP TRIGGER community_aggregates_subscriber_count ON community_follower;

DROP FUNCTION community_aggregates_community, community_aggregates_post_count, community_aggregates_comment_count, community_aggregates_subscriber_count;

