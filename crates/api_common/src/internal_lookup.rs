use crate::sensitive::Sensitive;
use lemmy_db_schema::newtypes::DbUrl;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InternalLookupRequest {
  // The actor_id to lookup.  This can be a Post or Comment actor_id based on the value of `lookup_type`.
  pub actor_id: DbUrl,
  // The type of the actor_id to lookup.
  pub lookup_type: InternalLookupType,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InternalLookupResponse {
  // Not sure how to abstract PostId and CommentId here without splitting this into two endpoints.
  pub internal_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub enum InternalLookupType {
  Post,
  Comment,
}
