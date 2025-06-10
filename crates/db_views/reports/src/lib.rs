use lemmy_db_schema::source::{
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
  lemmy_db_schema::{
    utils::queries::{
      comment_creator_is_admin,
      creator_community_actions_select,
      person1_select,
      person2_select,
      post_creator_is_admin,
    },
    CreatorCommunityActionsAllColumnsTuple,
    Person1AliasAllColumnsTuple,
    Person2AliasAllColumnsTuple,
  },
  ts_rs::TS,
};

pub mod api;
#[cfg(feature = "full")]
pub mod comment_report_view;

#[cfg(feature = "full")]
pub mod community_report_view;

#[cfg(feature = "full")]
pub mod post_report_view;

#[cfg(feature = "full")]
pub mod private_message_report_view;

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
