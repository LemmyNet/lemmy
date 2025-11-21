use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  PersonContentType,
  newtypes::PaginationCursor,
  source::{
    combined::person_content::PersonContentCombined,
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    images::ImageDetails,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    tag::TagsView,
  },
};
use lemmy_db_schema_file::PersonId;
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeBanExpiresType,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_admin,
    creator_is_moderator,
    creator_local_home_ban_expires,
    creator_local_home_banned,
    local_user_can_mod,
    post_tags_fragment,
  },
  lemmy_db_views_local_user::LocalUserView,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[cfg(feature = "full")]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// A combined person_content view
pub(crate) struct PersonContentCombinedViewInternal {
  #[diesel(embed)]
  pub person_content_combined: PersonContentCombined,
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
  #[diesel(
      select_expression = local_user_can_mod()
    )
  ]
  pub can_mod: bool,
  #[diesel(select_expression = creator_local_home_banned())]
  pub creator_banned: bool,
  #[
    diesel(
      select_expression_type = CreatorLocalHomeBanExpiresType,
      select_expression = creator_local_home_ban_expires()
     )
  ]
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
pub enum PersonContentCombinedView {
  Post(PostView),
  Comment(CommentView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets a person's content (posts and comments)
///
/// Either person_id, or username are required.
pub struct ListPersonContent {
  pub type_: Option<PersonContentType>,
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  pub username: Option<String>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person's content response.
pub struct ListPersonContentResponse {
  pub content: Vec<PersonContentCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
