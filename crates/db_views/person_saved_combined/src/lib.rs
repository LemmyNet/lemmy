use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  PersonContentType,
  newtypes::PaginationCursor,
  source::{
    combined::person_saved::PersonSavedCombined,
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    images::ImageDetails,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    tag::TagsView,
  },
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeCommunityBanExpiresType,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_admin,
    creator_is_moderator,
    creator_local_home_community_ban_expires,
    creator_local_home_community_banned,
    local_user_can_mod,
    post_tags_fragment,
  },

  lemmy_db_views_local_user::LocalUserView,
};

#[cfg(feature = "full")]
pub mod impls;

#[cfg(feature = "full")]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// A combined person_saved view
pub(crate) struct PersonSavedCombinedViewInternal {
  #[diesel(embed)]
  pub person_saved_combined: PersonSavedCombined,
  #[diesel(embed)]
  pub comment: Option<Comment>,
  #[diesel(embed)]
  pub post: Post,
  #[diesel(embed)]
  pub item_creator: Person,
  #[diesel(embed)]
  pub community: Community,
  #[diesel(embed)]
  pub community_actions: Option<CommunityActions>,
  #[diesel(embed)]
  pub post_actions: Option<PostActions>,
  #[diesel(embed)]
  pub person_actions: Option<PersonActions>,
  #[diesel(embed)]
  pub comment_actions: Option<CommentActions>,
  #[diesel(embed)]
  pub image_details: Option<ImageDetails>,
  #[diesel(select_expression = creator_is_admin())]
  pub item_creator_is_admin: bool,
  #[diesel(select_expression = post_tags_fragment())]
  pub post_tags: TagsView,
  #[diesel(select_expression = local_user_can_mod())]
  pub can_mod: bool,
  #[diesel(select_expression = creator_local_home_community_banned())]
  pub creator_banned: bool,
  #[diesel(
    select_expression_type = CreatorLocalHomeCommunityBanExpiresType,
    select_expression = creator_local_home_community_ban_expires()
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  #[diesel(select_expression = creator_is_moderator())]
  pub creator_is_moderator: bool,
  #[diesel(select_expression = creator_banned_from_community())]
  pub creator_banned_from_community: bool,
  #[diesel(select_expression = creator_ban_expires_from_community())]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum PersonSavedCombinedView {
  Post(PostView),
  Comment(CommentView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your saved posts and comments
pub struct ListPersonSaved {
  pub type_: Option<PersonContentType>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person's saved content response.
pub struct ListPersonSavedResponse {
  pub saved: Vec<PersonSavedCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
