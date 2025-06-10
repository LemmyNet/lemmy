use lemmy_db_schema::source::{instance::InstanceActions, person::Person};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{helper_types::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{
    utils::{
      functions::coalesce,
      queries::{
        creator_banned,
        creator_home_instance_actions_select,
        creator_local_instance_actions_select,
      },
    },
    CreatorHomeInstanceActionsAllColumnsTuple,
    CreatorLocalInstanceActionsAllColumnsTuple,
  },
  lemmy_db_schema_file::schema::local_user,
  ts_rs::TS,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
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
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorHomeInstanceActionsAllColumnsTuple>,
      select_expression = creator_home_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorLocalInstanceActionsAllColumnsTuple>,
      select_expression = creator_local_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned()
    )
  )]
  pub creator_banned: bool,
}
