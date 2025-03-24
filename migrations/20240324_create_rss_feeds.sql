-- Create RSS feed configurations table
CREATE TABLE rss_feeds (
    id SERIAL PRIMARY KEY,
    feed_url TEXT NOT NULL,
    community_id INTEGER NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    check_frequency_minutes INTEGER NOT NULL DEFAULT 60,
    last_check TIMESTAMP WITH TIME ZONE,
    last_item_guid TEXT,
    bot_account_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    content_transform_rules JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(feed_url, community_id)
);

-- Create index for faster lookups
CREATE INDEX idx_rss_feeds_community_id ON rss_feeds(community_id);
CREATE INDEX idx_rss_feeds_last_check ON rss_feeds(last_check);
CREATE INDEX idx_rss_feeds_active ON rss_feeds(last_check) 
WHERE is_active = true;

CREATE TYPE rss_feed_status AS ENUM ('success', 'error', 'skipped');

-- Create RSS feed processing history table
CREATE TABLE rss_feed_history (
    id SERIAL PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES rss_feeds(id) ON DELETE CASCADE,
    status rss_feed_status NOT NULL,
    error_message TEXT,
    items_processed INTEGER NOT NULL DEFAULT 0,
    processed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index for feed history
CREATE INDEX idx_rss_feed_history_feed_id ON rss_feed_history(feed_id);
CREATE INDEX idx_rss_feed_history_processed_at ON rss_feed_history(processed_at); 