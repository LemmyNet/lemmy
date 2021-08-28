use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    generate_activity_id,
    verify_activity,
    verify_person_in_community,
    voting::{vote_comment, vote_post},
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::{
    objects::get_or_fetch_and_insert_post_or_comment,
    person::get_or_fetch_and_upsert_person,
  },
  ActorType,
  PostOrComment,
};
use activitystreams::{base::AnyBase, primitives::OneOrMany, unparsed::Unparsed};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityFields, ActivityHandler};
use lemmy_db_queries::Crud;
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  CommunityId,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, ops::Deref};
use strum_macros::ToString;
use url::Url;

#[derive(Clone, Debug, ToString, Deserialize, Serialize)]
pub enum VoteType {
  Like,
  Dislike,
}

impl TryFrom<i16> for VoteType {
  type Error = LemmyError;

  fn try_from(value: i16) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(VoteType::Like),
      -1 => Ok(VoteType::Dislike),
      _ => Err(anyhow!("invalid vote value").into()),
    }
  }
}

impl From<&VoteType> for i16 {
  fn from(value: &VoteType) -> i16 {
    match value {
      VoteType::Like => 1,
      VoteType::Dislike => -1,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
  actor: Url,
  to: [PublicUrl; 1],
  pub(in crate::activities::voting) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  pub(in crate::activities::voting) kind: VoteType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl Vote {
  pub(in crate::activities::voting) fn new(
    object: &PostOrComment,
    actor: &Person,
    community: &Community,
    kind: VoteType,
  ) -> Result<Vote, LemmyError> {
    Ok(Vote {
      actor: actor.actor_id(),
      to: [PublicUrl::Public],
      object: object.ap_id(),
      cc: [community.actor_id()],
      kind: kind.clone(),
      id: generate_activity_id(kind)?,
      context: lemmy_context(),
      unparsed: Default::default(),
    })
  }

  pub async fn send(
    object: &PostOrComment,
    actor: &Person,
    community_id: CommunityId,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;
    let vote = Vote::new(object, actor, &community, kind)?;
    let vote_id = vote.id.clone();

    let activity = AnnouncableActivities::Vote(vote);
    send_to_community_new(activity, &vote_id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Vote {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_person_in_community(&self.actor, &self.cc[0], context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = get_or_fetch_and_upsert_person(&self.actor, context, request_counter).await?;
    let object =
      get_or_fetch_and_insert_post_or_comment(&self.object, context, request_counter).await?;
    match object {
      PostOrComment::Post(p) => vote_post(&self.kind, actor, p.deref(), context).await,
      PostOrComment::Comment(c) => vote_comment(&self.kind, actor, c.deref(), context).await,
    }
  }
}
