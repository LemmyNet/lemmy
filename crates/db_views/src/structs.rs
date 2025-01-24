#[cfg(feature = "full")]
use diesel::Queryable;
#[cfg(feature = "full")]
use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types};
use lemmy_db_schema::{
  aggregates::structs::{
    CommentAggregates,
    CommunityAggregates,
    PersonAggregates,
    PostAggregates,
    SiteAggregates,
  },
  source::{
    comment::Comment,
    comment_report::CommentReport,
    community::Community,
    community_report::CommunityReport,
    custom_emoji::CustomEmoji,
    custom_emoji_keyword::CustomEmojiKeyword,
    images::{ImageDetails, LocalImage},
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    local_user::LocalUser,
    local_user_vote_display_mode::LocalUserVoteDisplayMode,
    person::Person,
    post::Post,
    post_report::PostReport,
    private_message::PrivateMessage,
    private_message_report::PrivateMessageReport,
    registration_application::RegistrationApplication,
    site::Site,
    tag::Tag,
  },
  SubscribedType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment report view.
pub struct CommentReportView {
  pub comment_report: CommentReport,
  pub comment: Comment,
  pub post: Post,
  pub community: Community,
  pub creator: Person,
  pub comment_creator: Person,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub creator_blocked: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub resolver: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment view.
pub struct CommentView {
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
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
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community report view.
pub struct CommunityReportView {
  pub community_report: CommunityReport,
  pub community: Community,
  pub creator: Person,
  pub counts: CommunityAggregates,
  pub subscribed: SubscribedType,
  #[cfg_attr(feature = "full", ts(optional))]
  pub resolver: Option<Person>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local user view.
pub struct LocalUserView {
  pub local_user: LocalUser,
  pub local_user_vote_display_mode: LocalUserVoteDisplayMode,
  pub person: Person,
  pub counts: PersonAggregates,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A post report view.
pub struct PostReportView {
  pub post_report: PostReport,
  pub post: Post,
  pub community: Community,
  pub creator: Person,
  pub post_creator: Person,
  pub creator_banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub read: bool,
  pub hidden: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub unread_comments: i64,
  pub counts: PostAggregates,
  #[cfg_attr(feature = "full", ts(optional))]
  pub resolver: Option<Person>,
}

/// currently this is just a wrapper around post id, but should be seen as opaque from the client's
/// perspective. stringified since we might want to use arbitrary info later, with a P prepended to
/// prevent ossification (api users love to make assumptions (e.g. parse stuff that looks like
/// numbers as numbers) about apis that aren't part of the spec
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PaginationCursor(pub String);

/// like PaginationCursor but for the report_combined table
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ReportCombinedPaginationCursor(pub String);

/// like PaginationCursor but for the person_content_combined table
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PersonContentCombinedPaginationCursor(pub String);

/// like PaginationCursor but for the person_saved_combined table
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PersonSavedCombinedPaginationCursor(pub String);

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A post view.
pub struct PostView {
  pub post: Post,
  pub creator: Person,
  pub community: Community,
  #[cfg_attr(feature = "full", ts(optional))]
  pub image_details: Option<ImageDetails>,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub counts: PostAggregates,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub read: bool,
  pub hidden: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub unread_comments: i64,
  pub tags: PostTags,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message report view.
pub struct PrivateMessageReportView {
  pub private_message_report: PrivateMessageReport,
  pub private_message: PrivateMessage,
  pub private_message_creator: Person,
  pub creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  pub resolver: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A registration application view.
pub struct RegistrationApplicationView {
  pub registration_application: RegistrationApplication,
  pub creator_local_user: LocalUser,
  pub creator: Person,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A site view.
pub struct SiteView {
  pub site: Site,
  pub local_site: LocalSite,
  pub local_site_rate_limit: LocalSiteRateLimit,
  pub counts: SiteAggregates,
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
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local image view.
pub struct LocalImageView {
  pub local_image: LocalImage,
  pub person: Person,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined report view
pub struct ReportCombinedViewInternal {
  // Post-specific
  pub post_report: Option<PostReport>,
  pub post: Option<Post>,
  pub post_counts: Option<PostAggregates>,
  pub post_unread_comments: Option<i64>,
  pub post_saved: bool,
  pub post_read: bool,
  pub post_hidden: bool,
  pub my_post_vote: Option<i16>,
  // Comment-specific
  pub comment_report: Option<CommentReport>,
  pub comment: Option<Comment>,
  pub comment_counts: Option<CommentAggregates>,
  pub comment_saved: bool,
  pub my_comment_vote: Option<i16>,
  // Private-message-specific
  pub private_message_report: Option<PrivateMessageReport>,
  pub private_message: Option<PrivateMessage>,
  // Community-specific
  pub community_report: Option<CommunityReport>,
  pub community_counts: Option<CommunityAggregates>,
  // Shared
  pub report_creator: Person,
  pub item_creator: Option<Person>,
  pub community: Option<Community>,
  pub subscribed: SubscribedType,
  pub resolver: Option<Person>,
  pub item_creator_is_admin: bool,
  pub item_creator_banned_from_community: bool,
  pub item_creator_is_moderator: bool,
  pub item_creator_blocked: bool,
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
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined person_content view
pub struct PersonContentViewInternal {
  // Post-specific
  pub post_counts: PostAggregates,
  pub post_unread_comments: i64,
  pub post_saved: bool,
  pub post_read: bool,
  pub post_hidden: bool,
  pub my_post_vote: Option<i16>,
  pub image_details: Option<ImageDetails>,
  pub post_tags: PostTags,
  // Comment-specific
  pub comment: Option<Comment>,
  pub comment_counts: Option<CommentAggregates>,
  pub comment_saved: bool,
  pub my_comment_vote: Option<i16>,
  // Shared
  pub post: Post,
  pub community: Community,
  pub item_creator: Person,
  pub subscribed: SubscribedType,
  pub item_creator_is_admin: bool,
  pub item_creator_is_moderator: bool,
  pub item_creator_banned_from_community: bool,
  pub item_creator_blocked: bool,
  pub banned_from_community: bool,
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

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Default)]
#[cfg_attr(feature = "full", derive(TS, FromSqlRow, AsExpression))]
#[serde(transparent)]
#[cfg_attr(feature = "full", diesel(sql_type = Nullable<sql_types::Json>))]
/// we wrap this in a struct so we can implement FromSqlRow<Json> for it
pub struct PostTags {
  pub tags: Vec<Tag>,
}
