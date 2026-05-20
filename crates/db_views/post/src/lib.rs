use chrono::{DateTime, Utc};
use lemmy_db_schema::source::{
  community::{Community, CommunityActions},
  community_tag::CommunityTagsView,
  images::ImageDetails,
  person::{Person, PersonActions},
  post::{Post, PostActions},
};
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod db_perf;
#[cfg(test)]
mod test;
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::post_select_remove_deletes,
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeBanExpiresType,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_moderator,
    creator_local_home_ban_expires,
    creator_local_home_community_banned,
    local_user_can_mod_post,
    post_community_tags_fragment,
    post_creator_is_admin,
  },
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post view.
pub struct PostView {
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_select_remove_deletes()
    )
  )]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_community_tags_fragment()
    )
  )]
  pub tags: CommunityTagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod_post()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_local_home_community_banned()
    )
  )]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = CreatorLocalHomeBanExpiresType,
      select_expression = creator_local_home_ban_expires()
     )
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_moderator()
    )
  )]
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_community()
    )
  )]
  pub creator_banned_from_community: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_ban_expires_from_community()
    )
  )]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}
