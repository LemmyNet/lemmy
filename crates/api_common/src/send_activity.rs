use crate::{
  community::BanFromCommunity,
  context::LemmyContext,
  person::BanPerson,
  post::{DeletePost, RemovePost},
};
use activitypub_federation::config::Data;
use futures::future::BoxFuture;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, PersonId},
  source::{
    comment::Comment,
    community::Community,
    person::Person,
    post::Post,
    private_message::PrivateMessage,
  },
};
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_utils::error::LemmyResult;
use once_cell::sync::OnceCell;
use url::Url;

type MatchOutgoingActivitiesBoxed =
  Box<for<'a> fn(SendActivityData, &'a Data<LemmyContext>) -> BoxFuture<'a, LemmyResult<()>>>;

/// This static is necessary so that the api_common crates don't need to depend on lemmy_apub
pub static MATCH_OUTGOING_ACTIVITIES: OnceCell<MatchOutgoingActivitiesBoxed> = OnceCell::new();

#[derive(Debug)]
pub enum SendActivityData {
  CreatePost(Post),
  UpdatePost(Post),
  DeletePost(Post, Person, DeletePost),
  RemovePost(Post, Person, RemovePost),
  LockPost(Post, Person, bool),
  FeaturePost(Post, Person, bool),
  CreateComment(Comment),
  UpdateComment(Comment),
  DeleteComment(Comment, Person, Community),
  RemoveComment(Comment, Person, Community, Option<String>),
  LikePostOrComment(DbUrl, Person, Community, i16),
  FollowCommunity(Community, Person, bool),
  UpdateCommunity(Person, Community),
  DeleteCommunity(Person, Community, bool),
  RemoveCommunity(Person, Community, Option<String>, bool),
  AddModToCommunity(Person, CommunityId, PersonId, bool),
  BanFromCommunity(Person, CommunityId, Person, BanFromCommunity),
  BanFromSite(Person, Person, BanPerson),
  CreatePrivateMessage(PrivateMessageView),
  UpdatePrivateMessage(PrivateMessageView),
  DeletePrivateMessage(Person, PrivateMessage, bool),
  DeleteUser(Person, bool),
  CreateReport(Url, Person, Community, String),
}

pub struct ActivityChannel;

impl ActivityChannel {
  pub async fn submit_activity(
    data: SendActivityData,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    MATCH_OUTGOING_ACTIVITIES
      .get()
      .expect("retrieve function pointer")(data, context)
    .await
  }
}
