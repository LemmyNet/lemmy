use crate::models::{RssFeed, RssError, ContentTransformRules};
use feed_rs::parser;
use reqwest::Client;
use sqlx::PgPool;
use tracing::{info, error, warn};
use chrono::Utc;
use lemmy_db_schema::{
    source::{
        post::{Post, PostInsertForm},
        person::Person,
    },
    traits::Crud,
};
use lemmy_utils::error::LemmyResult;

pub struct FeedProcessor {
    db: PgPool,
    client: Client,
}

impl FeedProcessor {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            client: Client::new(),
        }
    }

    pub async fn process_feed(&self, feed: &RssFeed) -> Result<(), RssError> {
        info!("Processing feed: {}", feed.feed_url);

        // Fetch feed content
        let response = self.client
            .get(&feed.feed_url)
            .send()
            .await
            .map_err(|e| RssError::FetchError(e.to_string()))?;

        let content = response
            .bytes()
            .await
            .map_err(|e| RssError::FetchError(e.to_string()))?;

        // Parse feed
        let parsed_feed = parser::parse(&content[..])
            .map_err(|e| RssError::ParseError(e.to_string()))?;

        // Process entries
        let mut items_processed = 0;
        let mut newest_guid = None;
        
        for entry in parsed_feed.entries {
            // Skip if we've already processed this item
            if Some(entry.id.clone()) == feed.last_item_guid {
                break;
            }

            // Transform content based on rules
            let transformed_content = self.transform_content(&entry, &feed.content_transform_rules)?;

            // Create post in community
            self.create_post(feed, &transformed_content).await?;
            items_processed += 1;
            newest_guid = Some(entry.id);
        }

        // Update feed status
        self.update_feed_status(feed.id, items_processed, newest_guid).await?;

        Ok(())
    }

    fn transform_content(
        &self,
        entry: &feed_rs::model::Entry,
        rules: &Option<serde_json::Value>,
    ) -> Result<TransformedContent, RssError> {
        let rules = match rules {
            Some(rules_value) => serde_json::from_value(rules_value.clone())
                .map_err(|e| RssError::ConfigError(e.to_string()))?,
            None => ContentTransformRules::default(),
        };

        let title = rules.title_template
            .as_deref()
            .map(|template| template.replace("{title}", &entry.title.value))
            .unwrap_or_else(|| entry.title.value.clone());

        let content = match &entry.content {
            Some(content) => rules.content_template
                .as_deref()
                .map(|template| template.replace("{content}", &content.value))
                .unwrap_or_else(|| content.value.clone()),
            None => entry.summary.as_ref()
                .map(|summary| summary.value.clone())
                .unwrap_or_else(String::new),
        };

        let link = rules.link_template
            .as_deref()
            .map(|template| template.replace("{link}", &entry.links[0].href))
            .unwrap_or_else(|| entry.links[0].href.clone());

        Ok(TransformedContent {
            title,
            content,
            link,
            tags: rules.tags.unwrap_or_default(),
            custom_fields: rules.custom_fields,
        })
    }

    async fn create_post(&self, feed: &RssFeed, content: &TransformedContent) -> Result<(), RssError> {
        // Get the bot account
        let bot_account = match feed.bot_account_id {
            Some(id) => Person::read(&mut self.db, id)
                .await
                .map_err(|e| RssError::Database(e))?,
            None => return Err(RssError::ConfigError("No bot account configured for feed".into())),
        };

        // Create the post
        let post_form = PostInsertForm {
            name: content.title.clone(),
            body: Some(content.content.clone()),
            url: Some(content.link.clone()),
            creator_id: bot_account.id,
            community_id: feed.community_id,
            language_id: None, // Use default language
            ..Default::default()
        };

        Post::create(&mut self.db, &post_form)
            .await
            .map_err(|e| RssError::PostError(e.to_string()))?;

        info!("Created post in community {}: {}", feed.community_id, content.title);
        Ok(())
    }

    async fn update_feed_status(
        &self,
        feed_id: i32,
        items_processed: i32,
        newest_guid: Option<String>,
    ) -> Result<(), RssError> {
        sqlx::query!(
            r#"
            UPDATE rss_feeds
            SET last_check = $1,
                updated_at = $1,
                last_item_guid = COALESCE($3, last_item_guid)
            WHERE id = $2
            "#,
            Utc::now(),
            feed_id,
            newest_guid,
        )
        .execute(&self.db)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO rss_feed_history (feed_id, status, items_processed)
            VALUES ($1, 'success', $2)
            "#,
            feed_id,
            items_processed
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
struct TransformedContent {
    title: String,
    content: String,
    link: String,
    tags: Vec<String>,
    custom_fields: Option<serde_json::Value>,
} 