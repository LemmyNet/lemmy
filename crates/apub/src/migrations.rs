use serde::{Deserialize, Serialize};
use url::Url;

/// Migrate comment.in_reply_to field from containing post and parent comment ID, to only containing
/// the direct parent (whether its a post or comment). This is for compatibility with Pleroma and
/// Smithereen.
/// [https://github.com/LemmyNet/lemmy/issues/1454]
///
/// v0.12: receive both, send old (compatible with v0.11)
/// v0.13 receive both, send new (compatible with v0.12+, incompatible with v0.11)
/// v0.14: only send and receive new, remove migration (compatible with v0.13+)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CommentInReplyToMigration {
  Old(Vec<Url>),
  New(Url),
}
