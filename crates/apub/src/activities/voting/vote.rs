use crate::{
  activities::{
    community::{announce::GetCommunity, send_activity_in_community},
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
use activitystreams_kinds::public;
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{community::Community, post::Post, site::Site},
  traits::Crud,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

/// Vote has as:Public value in cc field, unlike other activities. This indicates to other software
/// (like GNU social, or presumably Mastodon), that the like actor should not be disclosed.
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
      to: vec![community.actor_id()],
      object: ObjectId::new(object.ap_id()),
      cc: vec![public()],
      kind: kind.clone(),
      id: generate_activity_id(kind, &context.settings().get_protocol_and_hostname())?,
      unparsed: Default::default(),
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    object: &PostOrComment,
    actor: &ApubPerson,
    community_id: CommunityId,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();
    let vote = Vote::new(object, actor, &community, kind, context)?;
    let vote_id = vote.id.clone();

    let activity = AnnouncableActivities::Vote(vote);
    send_activity_in_community(activity, &vote_id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Vote {
  type DataType = LemmyContext;

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    let site = blocking(context.pool(), Site::read_local_site).await??;
    if self.kind == VoteType::Dislike && !site.enable_downvotes {
      return Err(anyhow!("Downvotes disabled").into());
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self
      .actor
      .dereference(context, context.client(), request_counter)
      .await?;
    let object = self
      .object
      .dereference(context, context.client(), request_counter)
      .await?;
    match object {
      PostOrComment::Post(p) => vote_post(&self.kind, actor, &p, context).await,
      PostOrComment::Comment(c) => vote_comment(&self.kind, actor, &c, context).await,
    }
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for Vote {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let object = self
      .object
      .dereference(context, context.client(), request_counter)
      .await?;
    let cid = match object {
      PostOrComment::Post(p) => p.community_id,
      PostOrComment::Comment(c) => {
        blocking(context.pool(), move |conn| Post::read(conn, c.post_id))
          .await??
          .community_id
      }
    };
    let community = blocking(context.pool(), move |conn| Community::read(conn, cid)).await??;
    Ok(community.into())
  }
}
