use lemmy_db_schema::{
  aggregates::structs::{CommentAggregates, PersonAggregates, PostAggregates, SiteAggregates},
  source::{
    comment::Comment,
    comment_report::CommentReport,
    community::CommunitySafe,
    local_user::{LocalUser, LocalUserSettings},
    person::{Person, PersonSafe, PersonSafeAlias1, PersonSafeAlias2},
    post::Post,
    post_report::PostReport,
    private_message::PrivateMessage,
    registration_application::RegistrationApplication,
    site::Site,
  },
  SubscribedType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CommentReportView {
  pub comment_report: CommentReport,
  pub comment: Comment,
  pub post: Post,
  pub community: CommunitySafe,
  pub creator: PersonSafe,
  pub comment_creator: PersonSafeAlias1,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub my_vote: Option<i16>,                // Left join to CommentLike
  pub resolver: Option<PersonSafeAlias2>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CommentView {
  pub comment: Comment,
  pub creator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalUserSettingsView {
  pub local_user: LocalUserSettings,
  pub person: PersonSafe,
  pub counts: PersonAggregates,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PostReportView {
  pub post_report: PostReport,
  pub post: Post,
  pub community: CommunitySafe,
  pub creator: PersonSafe,
  pub post_creator: PersonSafeAlias1,
  pub creator_banned_from_community: bool,
  pub my_vote: Option<i16>,
  pub counts: PostAggregates,
  pub resolver: Option<PersonSafeAlias2>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PostView {
  pub post: Post,
  pub creator: PersonSafe,
  pub community: CommunitySafe,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub counts: PostAggregates,
  pub subscribed: SubscribedType, // Left join to CommunityFollower
  pub saved: bool,                // Left join to PostSaved
  pub read: bool,                 // Left join to PostRead
  pub creator_blocked: bool,      // Left join to PersonBlock
  pub my_vote: Option<i16>,       // Left join to PostLike
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PrivateMessageView {
  pub private_message: PrivateMessage,
  pub creator: PersonSafe,
  pub recipient: PersonSafeAlias1,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RegistrationApplicationView {
  pub registration_application: RegistrationApplication,
  pub creator_local_user: LocalUserSettings,
  pub creator: PersonSafe,
  pub admin: Option<PersonSafeAlias1>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteView {
  pub site: Site,
  pub counts: SiteAggregates,
}
