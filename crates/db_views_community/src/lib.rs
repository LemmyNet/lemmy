use lemmy_db_schema::source::{
  community::{Community, CommunityActions},
  instance::InstanceActions,
  tag::TagsView,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::{community_post_tags_fragment, local_user_community_can_mod},
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community view.
pub struct CommunityView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_community_can_mod()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_post_tags_fragment()
    )
  )]
  pub post_tags: TagsView,
}
