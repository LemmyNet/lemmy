use chrono::{DateTime, Utc};
use lemmy_db_schema::source::{
  comment::{Comment, CommentActions},
  community::{Community, CommunityActions},
  community_tag::CommunityTagsView,
  person::{Person, PersonActions},
  post::Post,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeCommunityBanExpiresType,
    comment_creator_is_admin,
    comment_select_remove_deletes,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_moderator,
    creator_local_home_community_ban_expires,
    creator_local_home_community_banned,
    local_user_can_mod_comment,
    post_community_tags_fragment,
  },
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A slimmer comment view, without the post, or community.
pub struct CommentSlimView {
  pub comment: Comment,
  pub creator: Person,
  pub comment_actions: Option<CommentActions>,
  pub person_actions: Option<PersonActions>,
  pub creator_is_admin: bool,
  pub can_mod: bool,
  pub creator_banned: bool,
  pub creator_is_moderator: bool,
  pub creator_banned_from_community: bool,
}
