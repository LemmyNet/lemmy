use crate::{
  activities::{
    community::{announce::GetCommunity, send_activity_in_community},
    generate_activity_id,
    verify_person_in_community,
    voting::{vote_comment, vote_post},
  },
  activity_lists::AnnouncableActivities,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::voting::vote::{Vote, VoteType},
  ActorType,
  PostOrComment,
};
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::public;
use anyhow::anyhow;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{community::Community, post::Post, site::Site},
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

/// Vote has as:Public value in cc field, unlike other activities. This indicates to other software
/// (like GNU social, or presumably Mastodon), that the like actor should not be disclosed.
impl Vote {
  pub(in crate::activities::voting) fn new(
    object: &PostOrComment,
    actor: &ApubPerson,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<Vote, LemmyError> {
    Ok(Vote {
      actor: ObjectId::new(actor.actor_id()),
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
    let vote = Vote::new(object, actor, kind, context)?;

    let activity = AnnouncableActivities::Vote(vote);
    send_activity_in_community(activity, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Vote {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
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
      .dereference(context, local_instance(context), request_counter)
      .await?;
    let object = self
      .object
      .dereference(context, local_instance(context), request_counter)
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
      .dereference(context, local_instance(context), request_counter)
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
