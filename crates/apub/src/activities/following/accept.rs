use crate::{
  activities::{generate_activity_id, send_lemmy_activity},
  local_instance,
  protocol::activities::following::{accept::AcceptFollowCommunity, follow::FollowCommunity},
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor},
  utils::verify_urls_match,
};
use activitystreams_kinds::activity::AcceptType;
use lemmy_api_common::{community::CommunityResponse, utils::blocking};
use lemmy_db_schema::{source::community::CommunityFollower, traits::Followable};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperation};
use url::Url;

impl AcceptFollowCommunity {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    follow: FollowCommunity,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = follow.object.dereference_local(context).await?;
    let person = follow
      .actor
      .clone()
      .dereference(context, local_instance(context), request_counter)
      .await?;
    let accept = AcceptFollowCommunity {
      actor: ObjectId::new(community.actor_id()),
      object: follow,
      kind: AcceptType::Accept,
      id: generate_activity_id(
        AcceptType::Accept,
        &context.settings().get_protocol_and_hostname(),
      )?,
      unparsed: Default::default(),
    };
    let inbox = vec![person.shared_inbox_or_inbox()];
    send_lemmy_activity(context, accept, &community, inbox, true).await
  }
}

/// Handle accepted follows
#[async_trait::async_trait(?Send)]
impl ActivityHandler for AcceptFollowCommunity {
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
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self
      .actor
      .dereference(context, local_instance(context), request_counter)
      .await?;
    let person = self
      .object
      .actor
      .dereference(context, local_instance(context), request_counter)
      .await?;
    // This will throw an error if no follow was requested
    let community_id = community.id;
    let person_id = person.id;
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow_accepted(conn, community_id, person_id)
    })
    .await??;

    // Send the Subscribed message over websocket
    // Re-read the community_view to get the new SubscribedType
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await??;

    // Get the local_user_id
    let local_recipient_id = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, person_id)
    })
    .await??
    .local_user
    .id;

    let response = CommunityResponse { community_view };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::FollowCommunity,
      response,
      local_recipient_id,
      websocket_id: None,
    });

    Ok(())
  }
}
