use lemmy_db_schema::source::{
  comment::Comment,
  community::Community,
  instance::Instance,
  modlog::Modlog,
  person::Person,
  post::Post,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{utils::queries::selects::person1_select, Person1AliasAllColumnsTuple},
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[skip_serializing_none]
pub struct ModlogView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub modlog: Modlog,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub moderator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub target_person: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_instance: Option<Instance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_comment: Option<Comment>,
}
