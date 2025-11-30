use chrono::{DateTime, Utc};
use lemmy_db_schema::source::{
  combined::report::ReportCombined,
  comment::{Comment, CommentActions},
  comment_report::CommentReport,
  community::{Community, CommunityActions},
  community_report::CommunityReport,
  person::{Person, PersonActions},
  post::{Post, PostActions},
  post_report::PostReport,
  private_message::PrivateMessage,
  private_message_report::PrivateMessageReport,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{NullableExpressionMethods, Queryable, Selectable, dsl::Nullable},
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeCommunityBanExpiresType,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_moderator,
    creator_local_home_community_ban_expires,
    creator_local_home_community_banned,
    local_user_is_admin,
    person1_select,
    person2_select,
  },
  lemmy_db_schema::{Person1AliasAllColumnsTuple, Person2AliasAllColumnsTuple},
  lemmy_db_views_local_user::LocalUserView,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[cfg(feature = "full")]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// A combined report view
pub struct ReportCombinedViewInternal {
  #[diesel(embed)]
  pub report_combined: ReportCombined,
  #[diesel(embed)]
  pub post_report: Option<PostReport>,
  #[diesel(embed)]
  pub comment_report: Option<CommentReport>,
  #[diesel(embed)]
  pub private_message_report: Option<PrivateMessageReport>,
  #[diesel(embed)]
  pub community_report: Option<CommunityReport>,
  #[diesel(
    select_expression_type = Person1AliasAllColumnsTuple,
    select_expression = person1_select()
  )]
  pub report_creator: Person,
  #[diesel(embed)]
  pub comment: Option<Comment>,
  #[diesel(embed)]
  pub private_message: Option<PrivateMessage>,
  #[diesel(embed)]
  pub post: Option<Post>,
  #[diesel(embed)]
  pub creator: Option<Person>,
  #[diesel(
    select_expression_type = Nullable<Person2AliasAllColumnsTuple>,
    select_expression = person2_select().nullable()
  )]
  pub resolver: Option<Person>,
  #[diesel(select_expression = local_user_is_admin())]
  pub creator_is_admin: bool,
  #[diesel(select_expression = creator_is_moderator())]
  pub creator_is_moderator: bool,
  #[diesel(select_expression = creator_local_home_community_banned())]
  pub creator_banned: bool,
  #[diesel(
    select_expression_type = CreatorLocalHomeCommunityBanExpiresType,
    select_expression = creator_local_home_community_ban_expires()
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  #[diesel(select_expression = creator_banned_from_community())]
  pub creator_banned_from_community: bool,
  #[diesel(select_expression = creator_ban_expires_from_community())]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
  #[diesel(embed)]
  pub community: Option<Community>,
  #[diesel(embed)]
  pub community_actions: Option<CommunityActions>,
  #[diesel(embed)]
  pub post_actions: Option<PostActions>,
  #[diesel(embed)]
  pub person_actions: Option<PersonActions>,
  #[diesel(embed)]
  pub comment_actions: Option<CommentActions>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum ReportCombinedView {
  Post(PostReportView),
  Comment(CommentReportView),
  PrivateMessage(PrivateMessageReportView),
  Community(CommunityReportView),
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A private message report view.
pub struct PrivateMessageReportView {
  pub private_message_report: PrivateMessageReport,
  pub private_message: PrivateMessage,
  pub creator: Person,
  pub private_message_creator: Person,
  pub resolver: Option<Person>,
  pub creator_is_admin: bool,
  pub creator_banned: bool,
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment report view.
pub struct CommentReportView {
  pub comment_report: CommentReport,
  pub comment: Comment,
  pub post: Post,
  pub community: Community,
  pub creator: Person,
  pub comment_creator: Person,
  pub comment_actions: Option<CommentActions>,
  pub resolver: Option<Person>,
  pub person_actions: Option<PersonActions>,
  pub community_actions: Option<CommunityActions>,
  pub creator_is_admin: bool,
  pub creator_is_moderator: bool,
  pub creator_banned: bool,
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  pub creator_banned_from_community: bool,
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A community report view.
pub struct CommunityReportView {
  pub community_report: CommunityReport,
  pub community: Community,
  pub creator: Person,
  pub resolver: Option<Person>,
  pub creator_is_admin: bool,
  pub creator_is_moderator: bool,
  pub creator_banned: bool,
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  pub creator_banned_from_community: bool,
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post report view.
pub struct PostReportView {
  pub post_report: PostReport,
  pub post: Post,
  pub community: Community,
  pub creator: Person,
  pub post_creator: Person,
  pub community_actions: Option<CommunityActions>,
  pub post_actions: Option<PostActions>,
  pub person_actions: Option<PersonActions>,
  pub resolver: Option<Person>,
  pub creator_is_admin: bool,
  pub creator_is_moderator: bool,
  pub creator_banned: bool,
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  pub creator_banned_from_community: bool,
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}
