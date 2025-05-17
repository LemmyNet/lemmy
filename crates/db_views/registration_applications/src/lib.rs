use lemmy_db_schema::source::{
  local_user::LocalUser,
  person::Person,
  registration_application::RegistrationApplication,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{helper_types::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{utils::queries::person1_select, Person1AliasAllColumnsTuple},
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A registration application view.
pub struct RegistrationApplicationView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub registration_application: RegistrationApplication,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator_local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub admin: Option<Person>,
}
