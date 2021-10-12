use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    generate_activity_id,
    verify_activity,
    verify_person_in_community,
    voting::{
      undo_vote_comment,
      undo_vote_post,
      vote::{Vote, VoteType},
    },
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::object_id::ObjectId,
  ActorType,
  PostOrComment,
};
use activitystreams::{
  activity::kind::UndoType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, verify_urls_match, ActivityFields, ActivityHandler};
use lemmy_db_queries::Crud;
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  CommunityId,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  actor: ObjectId<Person>,
  to: [PublicUrl; 1],
  object: Vote,
  cc: [ObjectId<Community>; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl UndoVote {
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

    let object = Vote::new(object, actor, &community, kind.clone(), context)?;
    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let undo_vote = UndoVote {
      actor: ObjectId::new(actor.actor_id()),
      to: [PublicUrl::Public],
      object,
      cc: [ObjectId::new(community.actor_id())],
      kind: UndoType::Undo,
      id: id.clone(),
      context: lemmy_context(),
      unparsed: Default::default(),
    };
    let activity = AnnouncableActivities::UndoVote(undo_vote);
    send_to_community_new(activity, &id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoVote {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self, &context.settings())?;
    verify_person_in_community(&self.actor, &self.cc[0], context, request_counter).await?;
    verify_urls_match(self.actor(), self.object.actor())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self.actor.dereference(context, request_counter).await?;
    let object = self
      .object
      .object
      .dereference(context, request_counter)
      .await?;
    match object {
      PostOrComment::Post(p) => undo_vote_post(actor, p.deref(), context).await,
      PostOrComment::Comment(c) => undo_vote_comment(actor, &c, context).await,
    }
  }
}
