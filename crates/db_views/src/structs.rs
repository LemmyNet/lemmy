use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{
  deserialize::FromSqlRow,
  dsl::exists,

  dsl::Nullable,
  expression::AsExpression,
  sql_types,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  PgExpressionMethods,
  QueryDsl,
  Queryable,
  Selectable,
};
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
    comment_reply::CommentReply,
    comment_report::CommentReport,
    community::Community,
    community_report::CommunityReport,
    custom_emoji::CustomEmoji,
    custom_emoji_keyword::CustomEmojiKeyword,
    images::{ImageDetails, LocalImage},
    instance::Instance,
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    local_user::LocalUser,
    local_user_vote_display_mode::LocalUserVoteDisplayMode,
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
        ModFeaturePost,
        ModHideCommunity,
        ModLockPost,
        ModRemoveComment,
        ModRemoveCommunity,
        ModRemovePost,
        ModTransferCommunity,
      },
    },
    person::Person,
    person_comment_mention::PersonCommentMention,
    person_post_mention::PersonPostMention,
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
#[cfg(feature = "full")]
use lemmy_db_schema::{
  aliases::{creator_community_actions, creator_local_user, person1},
  impls::comment::comment_select_remove_deletes,
  impls::community::community_follower_select_subscribed_type,
  impls::local_user::local_user_can_mod,
  schema::{comment, comment_actions, community_actions, local_user, person, person_actions},
  utils::functions::coalesce,
  Person1AliasAllColumnsTuple,
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
  #[cfg_attr(feature = "full", ts(optional))]
  /// The time when the comment was saved.
  pub saved: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub resolver: Option<Person>,
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
  pub counts: CommentAggregates,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression =
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null()
    )
  )]
  pub creator_banned_from_community: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression =
        community_actions::received_ban.nullable().is_not_null()
    )
  )]
  pub banned_from_community: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression =
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null()
    )
  )]
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression =
        exists(creator_local_user.filter(
          comment::creator_id
            .eq(creator_local_user.field(local_user::person_id))
            .and(creator_local_user.field(local_user::admin).eq(true)),
        ))
    )
  )]
  pub creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_follower_select_subscribed_type(),
    )
  )]
  pub subscribed: SubscribedType,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression =
        comment_actions::saved.nullable()
    )
  )]
  /// The time when the comment was saved.
  pub saved: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression =
        person_actions::blocked.nullable().is_not_null()
    )
  )]
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full",
    diesel(
      select_expression =
        comment_actions::like_score.nullable()
    )
  )]
  pub my_vote: Option<i16>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod()
    )
  )]
  pub can_mod: bool,
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
  #[cfg_attr(feature = "full", ts(optional))]
  /// The time when the comment was saved.
  pub saved: Option<DateTime<Utc>>,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub can_mod: bool,
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
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local user view.
pub struct LocalUserView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_user_vote_display_mode: LocalUserVoteDisplayMode,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
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
  #[cfg_attr(feature = "full", ts(optional))]
  /// The time when the post was saved.
  pub saved: Option<DateTime<Utc>>,
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
  #[cfg_attr(feature = "full", ts(optional))]
  /// The time when the post was saved.
  pub saved: Option<DateTime<Utc>>,
  pub read: bool,
  pub hidden: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub unread_comments: i64,
  pub tags: PostTags,
  pub can_mod: bool,
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
      select_expression = person1.fields(person::all_columns).nullable()
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
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined report view
pub struct ReportCombinedViewInternal {
  // Post-specific
  pub post_report: Option<PostReport>,
  pub post: Option<Post>,
  pub post_counts: Option<PostAggregates>,
  pub post_unread_comments: Option<i64>,
  pub post_saved: Option<DateTime<Utc>>,
  pub post_read: bool,
  pub post_hidden: bool,
  pub my_post_vote: Option<i16>,
  // Comment-specific
  pub comment_report: Option<CommentReport>,
  pub comment: Option<Comment>,
  pub comment_counts: Option<CommentAggregates>,
  pub comment_saved: Option<DateTime<Utc>>,
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
pub(crate) struct PersonContentCombinedViewInternal {
  // Post-specific
  pub post_counts: PostAggregates,
  pub post_unread_comments: i64,
  pub post_saved: Option<DateTime<Utc>>,
  pub post_read: bool,
  pub post_hidden: bool,
  pub my_post_vote: Option<i16>,
  pub image_details: Option<ImageDetails>,
  pub post_tags: PostTags,
  // Comment-specific
  pub comment: Option<Comment>,
  pub comment_counts: Option<CommentAggregates>,
  pub comment_saved: Option<DateTime<Utc>>,
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
  pub can_mod: bool,
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
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_follower_select_subscribed_type()
    )
  )]
  pub subscribed: SubscribedType,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_actions::blocked.nullable().is_not_null()
    )
  )]
  pub blocked: bool,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub counts: CommunityAggregates,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_actions::received_ban.nullable().is_not_null()
    )
  )]
  pub banned_from_community: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user::admin.nullable()
        .or(community_actions::became_moderator.nullable().is_not_null())
        .is_not_distinct_from(true)
    )
  )]
  pub can_mod: bool,
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
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub recipient: Person,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  #[cfg_attr(feature = "full", ts(optional))]
  /// The time when the comment was saved.
  pub saved: Option<DateTime<Utc>>,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub can_mod: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person post mention view.
pub struct PersonPostMentionView {
  pub person_post_mention: PersonPostMention,
  pub post: Post,
  pub creator: Person,
  pub community: Community,
  #[cfg_attr(feature = "full", ts(optional))]
  pub image_details: Option<ImageDetails>,
  pub recipient: Person,
  pub counts: PostAggregates,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  #[cfg_attr(feature = "full", ts(optional))]
  /// The time when the post was saved.
  pub saved: Option<DateTime<Utc>>,
  pub read: bool,
  pub hidden: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub unread_comments: i64,
  pub post_tags: PostTags,
  pub can_mod: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment reply view.
pub struct CommentReplyView {
  pub comment_reply: CommentReply,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub recipient: Person,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  #[cfg_attr(feature = "full", ts(optional))]
  /// The time when the comment was saved.
  pub saved: Option<DateTime<Utc>>,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub can_mod: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person view.
pub struct PersonView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub counts: PersonAggregates,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = coalesce<diesel::sql_types::Bool, Nullable<local_user::admin>, bool>,
      select_expression = coalesce(local_user::admin.nullable(), false)
    )
  )]
  pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PendingFollow {
  pub person: Person,
  pub community: Community,
  pub is_new_instance: bool,
  pub subscribed: SubscribedType,
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
      select_expression = person1.fields(person::all_columns)
    )
  )]
  pub recipient: Person,
}

/// like PaginationCursor but for the report_combined table
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InboxCombinedPaginationCursor(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined inbox view
pub struct InboxCombinedViewInternal {
  // Comment reply
  pub comment_reply: Option<CommentReply>,
  // Person comment mention
  pub person_comment_mention: Option<PersonCommentMention>,
  // Person post mention
  pub person_post_mention: Option<PersonPostMention>,
  pub post_counts: Option<PostAggregates>,
  pub post_unread_comments: Option<i64>,
  pub post_saved: Option<DateTime<Utc>>,
  pub post_read: bool,
  pub post_hidden: bool,
  pub my_post_vote: Option<i16>,
  pub image_details: Option<ImageDetails>,
  pub post_tags: PostTags,
  // Private message
  pub private_message: Option<PrivateMessage>,
  // Shared
  pub post: Option<Post>,
  pub community: Option<Community>,
  pub comment: Option<Comment>,
  pub comment_counts: Option<CommentAggregates>,
  pub comment_saved: Option<DateTime<Utc>>,
  pub my_comment_vote: Option<i16>,
  pub subscribed: SubscribedType,
  pub item_creator: Person,
  pub item_recipient: Person,
  pub item_creator_is_admin: bool,
  pub item_creator_is_moderator: bool,
  pub item_creator_banned_from_community: bool,
  pub item_creator_blocked: bool,
  pub banned_from_community: bool,
  pub can_mod: bool,
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
/// When a community is hidden from public view.
pub struct ModHideCommunityView {
  pub mod_hide_community: ModHideCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
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

/// like PaginationCursor but for the modlog_combined
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModlogCombinedPaginationCursor(pub String);

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
  pub mod_hide_community: Option<ModHideCommunity>,
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
      select_expression = person1.fields(person::all_columns).nullable()
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
  ModHideCommunity(ModHideCommunityView),
  ModLockPost(ModLockPostView),
  ModRemoveComment(ModRemoveCommentView),
  ModRemoveCommunity(ModRemoveCommunityView),
  ModRemovePost(ModRemovePostView),
  ModTransferCommunity(ModTransferCommunityView),
}

/// like PaginationCursor but for the modlog_combined
// TODO get rid of all these pagination cursors
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct SearchCombinedPaginationCursor(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined search view
pub(crate) struct SearchCombinedViewInternal {
  // Post-specific
  pub post: Option<Post>,
  pub post_counts: Option<PostAggregates>,
  pub post_unread_comments: Option<i64>,
  pub post_saved: Option<DateTime<Utc>>,
  pub post_read: bool,
  pub post_hidden: bool,
  pub my_post_vote: Option<i16>,
  pub image_details: Option<ImageDetails>,
  pub post_tags: PostTags,
  // // Comment-specific
  pub comment: Option<Comment>,
  pub comment_counts: Option<CommentAggregates>,
  pub comment_saved: Option<DateTime<Utc>>,
  pub my_comment_vote: Option<i16>,
  // // Community-specific
  pub community: Option<Community>,
  pub community_counts: Option<CommunityAggregates>,
  pub community_blocked: bool,
  pub subscribed: SubscribedType,
  // Person
  pub item_creator_counts: Option<PersonAggregates>,
  // Shared
  pub item_creator: Option<Person>,
  pub item_creator_is_admin: bool,
  pub item_creator_is_moderator: bool,
  pub item_creator_banned_from_community: bool,
  pub item_creator_blocked: bool,
  pub banned_from_community: bool,
  pub can_mod: bool,
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
#[cfg_attr(feature = "full", derive(TS, FromSqlRow, AsExpression))]
#[serde(transparent)]
#[cfg_attr(feature = "full", diesel(sql_type = Nullable<sql_types::Json>))]
/// we wrap this in a struct so we can implement FromSqlRow<Json> for it
pub struct PostTags {
  pub tags: Vec<Tag>,
}
