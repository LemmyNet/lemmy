-- Change the hot_ranks to a miniscule number, so that new / fetched content
-- won't crowd out existing content.
--
-- They must be non-zero, in order for them to be picked up by the hot_ranks updater.
-- See https://github.com/LemmyNet/lemmy/issues/4178
ALTER TABLE community_aggregates
    ALTER COLUMN hot_rank SET DEFAULT 0.0001;

ALTER TABLE comment_aggregates
    ALTER COLUMN hot_rank SET DEFAULT 0.0001;

ALTER TABLE post_aggregates
    ALTER COLUMN hot_rank SET DEFAULT 0.0001,
    ALTER COLUMN hot_rank_active SET DEFAULT 0.0001,
    ALTER COLUMN scaled_rank SET DEFAULT 0.0001;

