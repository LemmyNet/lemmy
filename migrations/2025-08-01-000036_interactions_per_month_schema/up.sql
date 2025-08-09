-- Add the interactions_month column
ALTER TABLE community_aggregates
    ADD COLUMN interactions_month bigint NOT NULL DEFAULT 0;

