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

// Another migration we are doing is to handle all deletions and removals using Delete activity.
// This is because Remove is for removing an object from a collection, so using it that way doesn't
// really make sense. It is also a problem because we have a RemoveMod activity, which was awkward
// to handle together with removing posts etc.
//
// v0.11: send and receive mod removals as Remove
// v0.12: receive removals as Remove, send as Delete (compatible with v0.11)
// v0.13: send and receive mod removals as Delete (compatible with v0.12)
//
// For v0.13, delete [`UndoRemovePostCommentOrCommunity`], and don't handle object deletion in
// [`RemoveMod`] handler.
