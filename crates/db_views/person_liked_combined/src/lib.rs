use lemmy_db_schema::{
  newtypes::PaginationCursor,
  source::{
    combined::person_liked::PersonLikedCombined,
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    images::ImageDetails,
    instance::InstanceActions,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    tag::TagsView,
  },
  LikeType,
  PersonContentType,
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::{
    creator_banned_from_community,
    creator_banned_within_community,
    creator_is_moderator,
  },
  lemmy_db_schema::utils::queries::{creator_is_admin, local_user_can_mod, post_tags_fragment},
  lemmy_db_views_local_user::LocalUserView,
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined person_saved view
pub(crate) struct PersonLikedCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_liked_combined: PersonLikedCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_admin()
    )
  )]
  pub item_creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_tags_fragment()
    )
  )]
  pub post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_within_community()
    )
  )]
  pub creator_banned: bool,
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
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum PersonLikedCombinedView {
  Post(PostView),
  Comment(CommentView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your liked / disliked posts
pub struct ListPersonLiked {
  pub type_: Option<PersonContentType>,
  pub like_type: Option<LikeType>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Your liked posts response.
pub struct ListPersonLikedResponse {
  pub liked: Vec<PersonLikedCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
