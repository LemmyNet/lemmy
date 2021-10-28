use crate::{
  activities::{
    community::{
      announce::{AnnouncableActivities, GetCommunity},
      send_to_community,
    },
    generate_activity_id,
    verify_activity,
    verify_is_public,
    verify_person_in_community,
    voting::{
      undo_vote_comment,
      undo_vote_post,
      vote::{Vote, VoteType},
    },
  },
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
  PostOrComment,
};
use activitystreams::{activity::kind::UndoType, public, unparsed::Unparsed};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler, ActorType},
  verify::verify_urls_match,
};
use lemmy_db_schema::{newtypes::CommunityId, source::community::Community, traits::Crud};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  actor: ObjectId<ApubPerson>,
  to: Vec<Url>,
  object: Vote,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: UndoType,
  id: Url,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl UndoVote {
  pub async fn send(
    object: &PostOrComment,
    actor: &ApubPerson,
    community_id: CommunityId,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community: ApubCommunity = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();

    let object = Vote::new(object, actor, &community, kind.clone(), context)?;
    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let undo_vote = UndoVote {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object,
      cc: vec![community.actor_id()],
      kind: UndoType::Undo,
      id: id.clone(),
      unparsed: Default::default(),
    };
    let activity = AnnouncableActivities::UndoVote(undo_vote);
    send_to_community(activity, &id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoVote {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to)?;
    verify_activity(self, &context.settings())?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_urls_match(self.actor(), self.object.actor())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
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

#[async_trait::async_trait(?Send)]
impl GetCommunity for UndoVote {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    self.object.get_community(context, request_counter).await
  }
}
