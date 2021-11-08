use crate::{
  activities::{
    community::{announce::GetCommunity, send_to_community},
    generate_activity_id,
    verify_activity,
    verify_is_public,
    verify_person_in_community,
    voting::{vote_comment, vote_post},
  },
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::voting::vote::{Vote, VoteType},
  PostOrComment,
};
use activitystreams::public;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

impl Vote {
  pub(in crate::activities::voting) fn new(
    object: &PostOrComment,
    actor: &ApubPerson,
    community: &ApubCommunity,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<Vote, LemmyError> {
    Ok(Vote {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: ObjectId::new(object.ap_id()),
      cc: vec![community.actor_id()],
      kind: kind.clone(),
      id: generate_activity_id(kind, &context.settings().get_protocol_and_hostname())?,
      unparsed: Default::default(),
    })
  }

  pub async fn send(
    object: &PostOrComment,
    actor: &ApubPerson,
    community_id: CommunityId,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community = context
      .conn()
      .await?
      .interact(move |conn| Community::read(conn, community_id))
      .await??
      .into();
    let vote = Vote::new(object, actor, &community, kind, context)?;
    let vote_id = vote.id.clone();

    let activity = AnnouncableActivities::Vote(vote);
    send_to_community(activity, &vote_id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Vote {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self.actor.dereference(context, request_counter).await?;
    let object = self.object.dereference(context, request_counter).await?;
    match object {
      PostOrComment::Post(p) => vote_post(&self.kind, actor, &p, context).await,
      PostOrComment::Comment(c) => vote_comment(&self.kind, actor, &c, context).await,
    }
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for Vote {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let object = self.object.dereference(context, request_counter).await?;
    let cid = match object {
      PostOrComment::Post(p) => p.community_id,
      PostOrComment::Comment(c) => {
        context
          .conn()
          .await?
          .interact(move |conn| Post::read(conn, c.post_id))
          .await??
          .community_id
      }
    };
    let community = context
      .conn()
      .await?
      .interact(move |conn| Community::read(conn, cid))
      .await??;
    Ok(community.into())
  }
}
