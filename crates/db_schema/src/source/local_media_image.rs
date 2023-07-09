// #[cfg(feature = "full")]
// use crate::schema::local_media_image;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
#[cfg_attr(feature = "full", ts(export))]
/// A local media image.
pub struct LocalMediaImage {
    // pub id: ImageId,
    pub source_url: String,
    pub local_url: String,
    pub token: String,
}