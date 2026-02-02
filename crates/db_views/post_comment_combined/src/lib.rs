use chrono::{DateTime, Utc};
use lemmy_db_schema::source::{
  comment::{Comment, CommentActions},
  community::{Community, CommunityActions},
  community_tag::CommunityTagsView,
  images::ImageDetails,
  person::{Person, PersonActions},
  post::{Post, PostActions},
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::traits::InternalToCombinedView,
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeCommunityBanExpiresType,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_admin,
    creator_is_moderator,
    creator_local_home_community_ban_expires,
    creator_local_home_community_banned,
    local_user_can_mod,
    post_community_tags_fragment,
  },
};

#[cfg(feature = "full")]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// A combined person_saved view
pub struct PostCommentCombinedViewInternal {
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
  #[diesel(select_expression = post_community_tags_fragment())]
  pub tags: CommunityTagsView,
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
pub enum PostCommentCombinedView {
  Post(PostView),
  Comment(CommentView),
}

#[cfg(feature = "full")]
impl InternalToCombinedView for PostCommentCombinedViewInternal {
  type CombinedView = PostCommentCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let Some(comment) = v.comment {
      Some(PostCommentCombinedView::Comment(CommentView {
        comment,
        post: v.post,
        community: v.community,
        creator: v.item_creator,
        community_actions: v.community_actions,
        comment_actions: v.comment_actions,
        person_actions: v.person_actions,
        creator_is_admin: v.item_creator_is_admin,
        tags: v.tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_ban_expires_at: v.creator_ban_expires_at,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_community: v.creator_banned_from_community,
        creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      }))
    } else {
      Some(PostCommentCombinedView::Post(PostView {
        post: v.post,
        community: v.community,
        creator: v.item_creator,
        image_details: v.image_details,
        community_actions: v.community_actions,
        post_actions: v.post_actions,
        person_actions: v.person_actions,
        creator_is_admin: v.item_creator_is_admin,
        tags: v.tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_ban_expires_at: v.creator_ban_expires_at,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_community: v.creator_banned_from_community,
        creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      }))
    }
  }
}

impl PostCommentCombinedView {
  /// Useful in combination with filter_map
  pub fn to_post_view(&self) -> Option<&PostView> {
    if let Self::Post(v) = self {
      Some(v)
    } else {
      None
    }
  }
}
