use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::Person1AliasAllColumnsTuple,
  lemmy_db_schema::utils::queries::selects::person1_select,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;
