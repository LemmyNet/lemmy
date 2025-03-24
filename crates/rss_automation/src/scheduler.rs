use crate::models::{RssFeed, RssError};
use crate::processor::FeedProcessor;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};

pub struct FeedScheduler {
    db: PgPool,
    processor: FeedProcessor,
}

impl FeedScheduler {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            processor: FeedProcessor::new(db.clone()),
        }
    }

    pub async fn start(&self) -> Result<(), RssError> {
        info!("Starting RSS feed scheduler");
        
        loop {
            if let Err(e) = self.process_due_feeds().await {
                error!("Error processing feeds: {}", e);
            }
            
            // Sleep for 1 minute before next check
            sleep(Duration::from_secs(60)).await;
        }
    }

    async fn process_due_feeds(&self) -> Result<(), RssError> {
        let now = Utc::now();
        
        // Get all active feeds that are due for processing
        let feeds = sqlx::query_as!(
            RssFeed,
            r#"
            SELECT * FROM rss_feeds
            WHERE is_active = true
            AND (
                last_check IS NULL
                OR last_check + (check_frequency_minutes * interval '1 minute') <= $1
            )
            "#,
            now
        )
        .fetch_all(&self.db)
        .await?;

        info!("Found {} feeds due for processing", feeds.len());

        // Process feeds concurrently with a limit
        let mut tasks = Vec::new();
        for feed in feeds {
            let feed_clone = feed.clone();
            let self_clone = self.clone();
            
            let task = tokio::spawn(async move {
                if let Err(e) = self_clone.process_feed(&feed_clone).await {
                    error!("Error processing feed {}: {}", feed_clone.id, e);
                    self_clone.record_failure(&feed_clone, &e.to_string()).await
                } else {
                    Ok(())
                }
            });
            
            tasks.push(task);
            
            // Limit concurrency
            if tasks.len() >= 5 {
                futures::future::join_all(tasks.drain(..)).await;
            }
        }
        
        // Wait for remaining tasks
        futures::future::join_all(tasks).await;
        
        Ok(())

    }

    async fn process_feed(&self, feed: &RssFeed) -> Result<(), RssError> {
        info!("Processing feed {}: {}", feed.id, feed.feed_url);
        
        match self.processor.process_feed(feed).await {
            Ok(_) => {
                info!("Successfully processed feed {}", feed.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to process feed {}: {}", feed.id, e);
                Err(e)
            }
        }
    }

    async fn record_failure(&self, feed: &RssFeed, error_message: &str) -> Result<(), RssError> {
        sqlx::query!(
            r#"
            INSERT INTO rss_feed_history (feed_id, status, error_message, items_processed)
            VALUES ($1, 'error', $2, 0)
            "#,
            feed.id,
            error_message
        )
        .execute(&self.db)
        .await?;
    
        Ok(())
    }
} 