use lemmy_db_schema::{
  aggregates::structs::{CommentAggregates, PersonAggregates, PostAggregates, SiteAggregates},
  source::{
    comment::Comment,
    comment_report::CommentReport,
    community::Community,
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    local_user::LocalUser,
    person::Person,
    post::Post,
    post_report::PostReport,
    private_message::PrivateMessage,
    private_message_report::PrivateMessageReport,
    registration_application::RegistrationApplication,
    site::Site,
  },
  SubscribedType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct CommentReportView {
  pub comment_report: CommentReport,
  pub comment: Comment,
  pub post: Post,
  pub community: Community,
  pub creator: Person,
  pub comment_creator: Person,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub my_vote: Option<i16>,                // Left join to CommentLike
  pub resolver: Option<Person>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct CommentView {
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub subscribed: SubscribedType,          // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub creator_blocked: bool,               // Left join to PersonBlock
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalUserView {
  pub local_user: LocalUser,
  pub person: Person,
  pub counts: PersonAggregates,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PostReportView {
  pub post_report: PostReport,
  pub post: Post,
  pub community: Community,
  pub creator: Person,
  pub post_creator: Person,
  pub creator_banned_from_community: bool,
  pub my_vote: Option<i16>,
  pub counts: PostAggregates,
  pub resolver: Option<Person>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PostView {
  pub post: Post,
  pub creator: Person,
  pub community: Community,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub counts: PostAggregates,
  pub subscribed: SubscribedType, // Left join to CommunityFollower
  pub saved: bool,                // Left join to PostSaved
  pub read: bool,                 // Left join to PostRead
  pub creator_blocked: bool,      // Left join to PersonBlock
  pub my_vote: Option<i16>,       // Left join to PostLike
  pub unread_comments: i64,       // Left join to PersonPostAggregates
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PrivateMessageView {
  pub private_message: PrivateMessage,
  pub creator: Person,
  pub recipient: Person,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PrivateMessageReportView {
  pub private_message_report: PrivateMessageReport,
  pub private_message: PrivateMessage,
  pub private_message_creator: Person,
  pub creator: Person,
  pub resolver: Option<Person>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct RegistrationApplicationView {
  pub registration_application: RegistrationApplication,
  pub creator_local_user: LocalUser,
  pub creator: Person,
  pub admin: Option<Person>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteView {
  pub site: Site,
  pub local_site: LocalSite,
  pub local_site_rate_limit: LocalSiteRateLimit,
  pub counts: SiteAggregates,
}
