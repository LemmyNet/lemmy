use lemmy_db_schema::source::{
  comment::{Comment, CommentActions},
  community::{Community, CommunityActions},
  instance::InstanceActions,
  person::{Person, PersonActions},
  post::Post,
  tag::TagsView,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{helper_types::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{
    utils::queries::{
      comment_creator_is_admin,
      comment_select_remove_deletes,
      creator_banned,
      creator_community_actions_select,
      creator_home_instance_actions_select,
      creator_local_instance_actions_select,
      local_user_can_mod,
      post_tags_fragment,
    },
    CreatorCommunityActionsAllColumnsTuple,
    CreatorHomeInstanceActionsAllColumnsTuple,
    CreatorLocalInstanceActionsAllColumnsTuple,
  },
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment view.
pub struct CommentView {
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = comment_select_remove_deletes()
    )
  )]
  pub comment: Comment,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorHomeInstanceActionsAllColumnsTuple>,
      select_expression = creator_home_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorLocalInstanceActionsAllColumnsTuple>,
      select_expression = creator_local_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<CreatorCommunityActionsAllColumnsTuple>,
      select_expression = creator_community_actions_select().nullable()
    )
  )]
  pub creator_community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = comment_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
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
      select_expression = creator_banned()
    )
  )]
  pub creator_banned: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A slimmer comment view, without the post, or community.
pub struct CommentSlimView {
  pub comment: Comment,
  pub creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_local_instance_actions: Option<InstanceActions>,
  pub creator_is_admin: bool,
  pub can_mod: bool,
  pub creator_banned: bool,
}
