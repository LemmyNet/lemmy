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
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::{
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_moderator,
    creator_local_home_community_ban_expires,
    creator_local_home_community_banned,
    local_user_is_admin,
    person1_select,
    person2_select,
    CreatorLocalHomeCommunityBanExpiresType,
  },
  lemmy_db_schema::{Person1AliasAllColumnsTuple, Person2AliasAllColumnsTuple},
  lemmy_db_views_local_user::LocalUserView,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

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
  pub creator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person2AliasAllColumnsTuple>,
      select_expression = person2_select().nullable()
    )
  )]
  pub resolver: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_is_admin()
    )
  )]
  pub creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_moderator()
    )
  )]
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_local_home_community_banned()
    )
  )]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = CreatorLocalHomeCommunityBanExpiresType,
      select_expression = creator_local_home_community_ban_expires()
     )
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_community()
    )
  )]
  pub creator_banned_from_community: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_ban_expires_from_community()
    )
  )]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
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
