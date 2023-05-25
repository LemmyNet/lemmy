use lemmy_db_schema::{
  aggregates::structs::{CommentAggregates, PersonAggregates, PostAggregates, SiteAggregates},
  source::{
    comment::Comment,
    comment_report::CommentReport,
    community::Community,
    custom_emoji::CustomEmoji,
    custom_emoji_keyword::CustomEmojiKeyword,
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
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
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
  pub my_vote: Option<i16>,
  pub resolver: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment view.
pub struct CommentView {
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub creator_blocked: bool,
  pub my_vote: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A local user view.
pub struct LocalUserView {
  pub local_user: LocalUser,
  pub person: Person,
  pub counts: PersonAggregates,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A post report view.
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

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A post view.
pub struct PostView {
  pub post: Post,
  pub creator: Person,
  pub community: Community,
  pub creator_banned_from_community: bool,
  pub counts: PostAggregates,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub read: bool,
  pub creator_blocked: bool,
  pub my_vote: Option<i16>,
  pub unread_comments: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message view.
pub struct PrivateMessageView {
  pub private_message: PrivateMessage,
  pub creator: Person,
  pub recipient: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message report view.
pub struct PrivateMessageReportView {
  pub private_message_report: PrivateMessageReport,
  pub private_message: PrivateMessage,
  pub private_message_creator: Person,
  pub creator: Person,
  pub resolver: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A registration application view.
pub struct RegistrationApplicationView {
  pub registration_application: RegistrationApplication,
  pub creator_local_user: LocalUser,
  pub creator: Person,
  pub admin: Option<Person>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A site view.
pub struct SiteView {
  pub site: Site,
  pub local_site: LocalSite,
  pub local_site_rate_limit: LocalSiteRateLimit,
  pub counts: SiteAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A custom emoji view.
pub struct CustomEmojiView {
  pub custom_emoji: CustomEmoji,
  pub keywords: Vec<CustomEmojiKeyword>,
}
