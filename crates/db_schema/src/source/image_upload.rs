use crate::newtypes::{ImageUploadId, LocalUserId};
#[cfg(feature = "full")]
use crate::schema::image_upload;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = image_upload))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::local_user::LocalUser))
)]
pub struct ImageUpload {
  pub id: ImageUploadId,
  pub local_user_id: LocalUserId,
  pub pictrs_alias: String,
  pub pictrs_delete_token: String,
  pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = image_upload))]
pub struct ImageUploadForm {
  pub local_user_id: LocalUserId,
  pub pictrs_alias: String,
  pub pictrs_delete_token: String,
}
