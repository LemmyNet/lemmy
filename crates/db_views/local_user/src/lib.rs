use lemmy_db_schema::source::{local_user::LocalUser, person::Person};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::creator_home_banned,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A local user view.
pub struct LocalUserView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_home_banned()
    )
  )]
  pub banned: bool,
}
