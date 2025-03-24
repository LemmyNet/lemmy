use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct RssFeed {
    pub id: i32,
    pub feed_url: String,
    pub community_id: i32,
    pub check_frequency_minutes: i32,
    pub last_check: Option<DateTime<Utc>>,
    pub last_item_guid: Option<String>,
    pub bot_account_id: Option<i32>,
    pub content_transform_rules: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rss_feed_status", rename_all = "lowercase")]
pub enum RssFeedStatus {
    Success,
    Error,
    Skipped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RssFeedHistory {
    pub id: i32,
    pub feed_id: i32,
    pub status: RssFeedStatus,
    pub error_message: Option<String>,
    pub items_processed: i32,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentTransformRules {
    pub title_template: Option<String>,
    pub content_template: Option<String>,
    pub link_template: Option<String>,
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
}

impl Default for ContentTransformRules {
    fn default() -> Self {
        Self {
            title_template: None,
            content_template: None,
            link_template: None,
            tags: None,
            custom_fields: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RssError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Feed fetch error: {0}")]
    FetchError(String),
    
    #[error("Feed parse error: {0}")]
    ParseError(String),
    
    #[error("Post creation error: {0}")]
    PostError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
} 