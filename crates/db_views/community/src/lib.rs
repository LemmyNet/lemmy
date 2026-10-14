use lemmy_db_schema::source::{
  community::{Community, CommunityActions},
  community_tag::CommunityTagsView,
  multi_community::MultiCommunity,
  person::Person,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::{
    community_tags_fragment,
    local_user_community_can_mod,
  },
  lemmy_db_schema_file::schema::multi_community_follow,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A community view.
pub struct CommunityView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_community_can_mod()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_tags_fragment()
    )
  )]
  pub tags: CommunityTagsView,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct MultiCommunityView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub multi: MultiCommunity,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = multi_community_follow::follow_state.nullable()
    )
  )]
  pub follow_state: Option<CommunityFollowerState>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub owner: Person,
}
