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
use lemmy_db_views_reports::{
  CommentReportView,
  CommunityReportView,
  PostReportView,
  PrivateMessageReportView,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{
    utils::queries::{local_user_is_admin, person1_select, person2_select},
    Person1AliasAllColumnsTuple,
    Person2AliasAllColumnsTuple,
  },
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
