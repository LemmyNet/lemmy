use chrono::{DateTime, Utc};
use lemmy_db_schema::source::person::{Person, PersonActions};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{NullableExpressionMethods, Queryable, Selectable, helper_types::Nullable},
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeBanExpiresType,
    creator_local_home_ban_expires,
    creator_local_home_banned,
  },
  lemmy_db_schema_file::schema::local_user,
  lemmy_diesel_utils::utils::functions::coalesce,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person view.
pub struct PersonView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = coalesce<diesel::sql_types::Bool, Nullable<local_user::admin>, bool>,
      select_expression = coalesce(local_user::admin.nullable(), false)
    )
  )]
  pub is_admin: bool,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_local_home_banned()
    )
  )]
  pub banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = CreatorLocalHomeBanExpiresType,
      select_expression = creator_local_home_ban_expires()
     )
  )]
  pub ban_expires_at: Option<DateTime<Utc>>,
}
