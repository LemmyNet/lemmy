use lemmy_db_schema::source::{
  combined::inbox::InboxCombined,
  comment::{Comment, CommentActions},
  comment_reply::CommentReply,
  community::{Community, CommunityActions},
  images::ImageDetails,
  instance::InstanceActions,
  person::{Person, PersonActions},
  person_comment_mention::PersonCommentMention,
  person_post_mention::PersonPostMention,
  post::{Post, PostActions},
  private_message::PrivateMessage,
  tag::TagsView,
};
use lemmy_db_views_private_message::PrivateMessageView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{
    utils::queries::{
      creator_banned,
      creator_community_actions_select,
      creator_home_instance_actions_select,
      creator_is_admin,
      creator_local_instance_actions_select,
      local_user_can_mod,
      person1_select,
      post_tags_fragment,
    },
    CreatorCommunityActionsAllColumnsTuple,
    CreatorHomeInstanceActionsAllColumnsTuple,
    CreatorLocalInstanceActionsAllColumnsTuple,
    Person1AliasAllColumnsTuple,
  },
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined inbox view
pub struct InboxCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub inbox_combined: InboxCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_reply: Option<CommentReply>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_comment_mention: Option<PersonCommentMention>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_post_mention: Option<PersonPostMention>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: Option<PrivateMessage>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub item_recipient: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<CreatorCommunityActionsAllColumnsTuple>,
      select_expression = creator_community_actions_select().nullable()
    )
  )]
  pub creator_community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorHomeInstanceActionsAllColumnsTuple>,
      select_expression = creator_home_instance_actions_select()))]
  pub creator_home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorLocalInstanceActionsAllColumnsTuple>,
      select_expression = creator_local_instance_actions_select()))]
  pub creator_local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
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
      select_expression = creator_banned()
    )
  )]
  pub creator_banned: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum InboxCombinedView {
  CommentReply(CommentReplyView),
  CommentMention(PersonCommentMentionView),
  PostMention(PersonPostMentionView),
  PrivateMessage(PrivateMessageView),
}
#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person comment mention view.
pub struct PersonCommentMentionView {
  pub person_comment_mention: PersonCommentMention,
  pub recipient: Person,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_community_actions: Option<CommunityActions>,
  pub creator_is_admin: bool,
  pub can_mod: bool,
  pub creator_banned: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person post mention view.
pub struct PersonPostMentionView {
  pub person_post_mention: PersonPostMention,
  pub recipient: Person,
  pub post: Post,
  pub creator: Person,
  pub community: Community,
  #[cfg_attr(feature = "full", ts(optional))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_community_actions: Option<CommunityActions>,
  pub post_tags: TagsView,
  pub creator_is_admin: bool,
  pub can_mod: bool,
  pub creator_banned: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment reply view.
pub struct CommentReplyView {
  pub comment_reply: CommentReply,
  pub recipient: Person,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_community_actions: Option<CommunityActions>,
  pub creator_is_admin: bool,
  pub post_tags: TagsView,
  pub can_mod: bool,
  pub creator_banned: bool,
}
