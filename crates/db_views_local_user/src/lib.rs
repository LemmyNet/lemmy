use lemmy_db_schema::source::{instance::InstanceActions, local_user::LocalUser, person::Person};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{dsl::Nullable, Queryable, Selectable},
  lemmy_db_schema::{
    utils::queries::creator_home_instance_actions_select,
    CreatorHomeInstanceActionsAllColumnsTuple,
  },
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local user view.
pub struct LocalUserView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorHomeInstanceActionsAllColumnsTuple>,
      select_expression = creator_home_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
}
