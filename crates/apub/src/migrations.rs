use lemmy_apub_lib::values::PublicUrl;
use serde::{Deserialize, Serialize};

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

/// Migrate value of field `to` from single value to vec.
///
/// v0.14: send as single value, accept both
/// v0.15: send as vec, accept both
/// v0.16: send and accept only vec
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PublicUrlMigration {
  Old(PublicUrl),
  New([PublicUrl; 1]),
}

impl PublicUrlMigration {
  pub(crate) fn create() -> PublicUrlMigration {
    PublicUrlMigration::Old(PublicUrl::Public)
  }
}
