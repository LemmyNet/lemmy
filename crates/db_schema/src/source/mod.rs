use lemmy_diesel_utils::dburl::DbUrl;
use url::Url;

#[cfg(feature = "full")]
pub mod activity;
pub mod actor_language;
pub mod captcha_answer;
pub mod combined;
pub mod comment;
pub mod comment_report;
pub mod community;
#[cfg(feature = "full")]
pub mod community_community_follow;
pub mod community_report;
pub mod community_tag;
pub mod custom_emoji;
pub mod custom_emoji_keyword;
pub mod email_verification;
pub mod federation_allowlist;
pub mod federation_blocklist;
pub mod federation_queue_state;
pub mod images;
pub mod instance;
pub mod keyword_block;
pub mod language;
pub mod local_site;
pub mod local_site_rate_limit;
pub mod local_site_url_blocklist;
pub mod local_user;
pub mod login_token;
pub mod modlog;
pub mod multi_community;
pub mod notification;
pub mod oauth_account;
pub mod oauth_provider;
pub mod password_reset_request;
pub mod person;
pub mod post;
pub mod post_report;
pub mod private_message;
pub mod private_message_report;
pub mod registration_application;
pub mod secret;
pub mod site;
pub mod tagline;

/// Default value for columns like [community::Community.inbox_url] which are marked as serde(skip).
///
/// This is necessary so they can be successfully deserialized from API responses, even though the
/// value is not sent by Lemmy. Necessary for crates which rely on Rust API such as
/// lemmy-stats-crawler.
#[allow(clippy::expect_used)]
fn placeholder_apub_url() -> DbUrl {
  DbUrl(Box::new(
    Url::parse("http://example.com").expect("parse placeholder url"),
  ))
}
