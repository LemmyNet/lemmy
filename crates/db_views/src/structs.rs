#[cfg(feature = "full")]
use crate::utils::{
  comment_creator_is_admin,
  comment_select_remove_deletes,
  community_post_tags_fragment,
  creator_banned,
  creator_community_actions_select,
  creator_home_instance_actions_select,
  creator_is_admin,
  creator_local_instance_actions_select,
  local_user_can_mod,
  local_user_community_can_mod,
  local_user_is_admin,
  person1_select,
  person2_select,
  post_creator_is_admin,
  post_tags_fragment,
};
#[cfg(feature = "full")]
use diesel::{
  deserialize::FromSqlRow,
  dsl::Nullable,
  expression::AsExpression,
  sql_types,
  NullableExpressionMethods,
  Queryable,
  Selectable,
};
use lemmy_db_schema::source::{
  combined::{
    inbox::InboxCombined,
    person_content::PersonContentCombined,
    person_saved::PersonSavedCombined,
    report::ReportCombined,
    search::SearchCombined,
  },
  comment::{Comment, CommentActions},
  comment_reply::CommentReply,
  comment_report::CommentReport,
  community::{Community, CommunityActions},
  community_report::CommunityReport,
  custom_emoji::CustomEmoji,
  custom_emoji_keyword::CustomEmojiKeyword,
  images::{ImageDetails, LocalImage},
  instance::{Instance, InstanceActions},
  local_site::LocalSite,
  local_site_rate_limit::LocalSiteRateLimit,
  local_user::LocalUser,
  mod_log::{
    admin::{
      AdminAllowInstance,
      AdminBlockInstance,
      AdminPurgeComment,
      AdminPurgeCommunity,
      AdminPurgePerson,
      AdminPurgePost,
    },
    moderator::{
      ModAdd,
      ModAddCommunity,
      ModBan,
      ModBanFromCommunity,
      ModChangeCommunityVisibility,
      ModFeaturePost,
      ModLockPost,
      ModRemoveComment,
      ModRemoveCommunity,
      ModRemovePost,
      ModTransferCommunity,
    },
  },
  person::{Person, PersonActions},
  person_comment_mention::PersonCommentMention,
  person_post_mention::PersonPostMention,
  post::{Post, PostActions},
  post_report::PostReport,
  private_message::PrivateMessage,
  private_message_report::PrivateMessageReport,
  registration_application::RegistrationApplication,
  site::Site,
  tag::Tag,
};
#[cfg(feature = "full")]
use lemmy_db_schema::{
  utils::functions::coalesce,
  CreatorCommunityActionsAllColumnsTuple,
  CreatorHomeInstanceActionsAllColumnsTuple,
  CreatorLocalInstanceActionsAllColumnsTuple,
  Person1AliasAllColumnsTuple,
  Person2AliasAllColumnsTuple,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::local_user;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment report view.
pub struct CommentReportView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_report: CommentReport,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Comment,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub comment_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person2AliasAllColumnsTuple>,
      select_expression = person2_select().nullable()
    )
  )]
  pub resolver: Option<Person>,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<CreatorCommunityActionsAllColumnsTuple>,
      select_expression = creator_community_actions_select().nullable()
    )
  )]
  pub creator_community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = comment_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
}

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

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community report view.
pub struct CommunityReportView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_report: CommunityReport,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person2AliasAllColumnsTuple>,
      select_expression = person2_select().nullable()
    )
  )]
  pub resolver: Option<Person>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local user view.
pub struct LocalUserView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorHomeInstanceActionsAllColumnsTuple>,
      select_expression = creator_home_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub instance_actions: Option<InstanceActions>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A post report view.
pub struct PostReportView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_report: PostReport,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub post_creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<CreatorCommunityActionsAllColumnsTuple>,
      select_expression = creator_community_actions_select().nullable()
    )
  )]
  pub creator_community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person2AliasAllColumnsTuple>,
      select_expression = person2_select().nullable()
    )
  )]
  pub resolver: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
}

/// currently this is just a wrapper around post id, but should be seen as opaque from the client's
/// perspective. stringified since we might want to use arbitrary info later, with a P prepended to
/// prevent ossification (api users love to make assumptions (e.g. parse stuff that looks like
/// numbers as numbers) about apis that aren't part of the spec
// TODO this is a mess, get rid of it and prefer the one in db_schema
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PostPaginationCursor(pub String);

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A post view.
pub struct PostView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_actions: Option<PostActions>,
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
      select_expression = post_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_tags_fragment()
    )
  )]
  pub tags: TagsView,
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
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message report view.
pub struct PrivateMessageReportView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message_report: PrivateMessageReport,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: PrivateMessage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub private_message_creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person2AliasAllColumnsTuple>,
      select_expression = person2_select().nullable()
    )
  )]
  pub resolver: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A registration application view.
pub struct RegistrationApplicationView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub registration_application: RegistrationApplication,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator_local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub admin: Option<Person>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A site view.
pub struct SiteView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub site: Site,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_site: LocalSite,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_site_rate_limit: LocalSiteRateLimit,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance: Instance,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A custom emoji view.
pub struct CustomEmojiView {
  pub custom_emoji: CustomEmoji,
  pub keywords: Vec<CustomEmojiKeyword>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A vote view for checking a post or comments votes.
pub struct VoteView {
  pub creator: Person,
  pub creator_banned_from_community: bool,
  pub score: i16,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local image view.
pub struct LocalImageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_image: LocalImage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined report view
pub struct ReportCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub report_combined: ReportCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_report: Option<PostReport>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_report: Option<CommentReport>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message_report: Option<PrivateMessageReport>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_report: Option<CommunityReport>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub report_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: Option<PrivateMessage>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub item_creator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person2AliasAllColumnsTuple>,
      select_expression = person2_select().nullable()
    )
  )]
  pub resolver: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
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
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_is_admin()
    )
  )]
  pub item_creator_is_admin: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum ReportCombinedView {
  Post(PostReportView),
  Comment(CommentReportView),
  PrivateMessage(PrivateMessageReportView),
  Community(CommunityReportView),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined person_content view
pub(crate) struct PersonContentCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_content_combined: PersonContentCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
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
pub enum PersonContentCombinedView {
  Post(PostView),
  Comment(CommentView),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined person_saved view
pub(crate) struct PersonSavedCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_saved_combined: PersonSavedCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
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
pub enum PersonSavedCombinedView {
  Post(PostView),
  Comment(CommentView),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community follower.
pub struct CommunityFollowerView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub follower: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community moderator.
pub struct CommunityModeratorView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub moderator: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A community person ban.
pub struct CommunityPersonBanView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
}

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

/// The community sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub enum CommunitySortType {
  ActiveSixMonths,
  #[default]
  ActiveMonthly,
  ActiveWeekly,
  ActiveDaily,
  Hot,
  New,
  Old,
  NameAsc,
  NameDesc,
  Comments,
  Posts,
  Subscribers,
  SubscribersLocal,
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person view.
pub struct PersonView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = coalesce<diesel::sql_types::Bool, Nullable<local_user::admin>, bool>,
      select_expression = coalesce(local_user::admin.nullable(), false)
    )
  )]
  pub is_admin: bool,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorHomeInstanceActionsAllColumnsTuple>,
      select_expression = creator_home_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorLocalInstanceActionsAllColumnsTuple>,
      select_expression = creator_local_instance_actions_select()))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned()
    )
  )]
  pub creator_banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PendingFollow {
  pub person: Person,
  pub community: Community,
  pub is_new_instance: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub follow_state: Option<CommunityFollowerState>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message view.
pub struct PrivateMessageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: PrivateMessage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub recipient: Person,
}

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
/// When someone is added as a community moderator.
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a site moderator.
pub struct ModAddView {
  pub mod_add: ModAdd,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from a community.
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from the site.
pub struct ModBanView {
  pub mod_ban: ModBan,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When the visibility of a community is changed
pub struct ModChangeCommunityVisibilityView {
  pub mod_change_community_visibility: ModChangeCommunityVisibility,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a comment.
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub comment: Comment,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a community.
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a post.
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePostView {
  pub mod_feature_post: ModFeaturePost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator transfers a community to a new owner.
pub struct ModTransferCommunityView {
  pub mod_transfer_community: ModTransferCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a comment.
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
  pub post: Post,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a community.
pub struct AdminPurgeCommunityView {
  pub admin_purge_community: AdminPurgeCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a person.
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminBlockInstanceView {
  pub admin_block_instance: AdminBlockInstance,
  pub instance: Instance,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminAllowInstanceView {
  pub admin_allow_instance: AdminAllowInstance,
  pub instance: Instance,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined modlog view
pub(crate) struct ModlogCombinedViewInternal {
  // Specific
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_allow_instance: Option<AdminAllowInstance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_block_instance: Option<AdminBlockInstance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_comment: Option<AdminPurgeComment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_community: Option<AdminPurgeCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_person: Option<AdminPurgePerson>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_post: Option<AdminPurgePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_add: Option<ModAdd>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_add_community: Option<ModAddCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_ban: Option<ModBan>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_ban_from_community: Option<ModBanFromCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_feature_post: Option<ModFeaturePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_change_community_visibility: Option<ModChangeCommunityVisibility>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_lock_post: Option<ModLockPost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_comment: Option<ModRemoveComment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_community: Option<ModRemoveCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_post: Option<ModRemovePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_transfer_community: Option<ModTransferCommunity>,
  // Specific fields

  // Shared
  #[cfg_attr(feature = "full", diesel(embed))]
  pub moderator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub other_person: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance: Option<Instance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum ModlogCombinedView {
  AdminAllowInstance(AdminAllowInstanceView),
  AdminBlockInstance(AdminBlockInstanceView),
  AdminPurgeComment(AdminPurgeCommentView),
  AdminPurgeCommunity(AdminPurgeCommunityView),
  AdminPurgePerson(AdminPurgePersonView),
  AdminPurgePost(AdminPurgePostView),
  ModAdd(ModAddView),
  ModAddCommunity(ModAddCommunityView),
  ModBan(ModBanView),
  ModBanFromCommunity(ModBanFromCommunityView),
  ModFeaturePost(ModFeaturePostView),
  ModChangeCommunityVisibility(ModChangeCommunityVisibilityView),
  ModLockPost(ModLockPostView),
  ModRemoveComment(ModRemoveCommentView),
  ModRemoveCommunity(ModRemoveCommunityView),
  ModRemovePost(ModRemovePostView),
  ModTransferCommunity(ModTransferCommunityView),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined search view
pub(crate) struct SearchCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub search_combined: SearchCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
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
  /// tags of this post
  pub post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_post_tags_fragment()
    )
  )]
  /// available tags in this community
  pub community_post_tags: TagsView,
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
pub enum SearchCombinedView {
  Post(PostView),
  Comment(CommentView),
  Community(CommunityView),
  Person(PersonView),
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "full", derive(TS, FromSqlRow, AsExpression))]
#[cfg_attr(feature = "full", diesel(sql_type = Nullable<sql_types::Json>))]
/// we wrap this in a struct so we can implement FromSqlRow<Json> for it
pub struct TagsView(pub Vec<Tag>);
