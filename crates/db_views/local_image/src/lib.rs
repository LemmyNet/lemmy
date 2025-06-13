#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use lemmy_db_schema::source::{images::LocalImage, person::Person, post::Post};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A local image view.
pub struct LocalImageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_image: LocalImage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
}
