use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::person1_select,
  lemmy_db_schema::Person1AliasAllColumnsTuple,
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message view.
pub struct PrivateMessageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: PrivateMessage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub recipient: Person,
}
