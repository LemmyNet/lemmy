use crate::{
  activities::{
    check_community_deleted_or_removed,
    comment::{collect_non_local_mentions, get_notif_recipients},
    community::{announce::GetCommunity, send_to_community},
    generate_activity_id,
    verify_activity,
    verify_is_public,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson},
  protocol::activities::{create_or_update::comment::CreateOrUpdateComment, CreateOrUpdateType},
};
use activitystreams::public;
use lemmy_api_common::{blocking, check_post_deleted_or_removed};
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType, ApubObject},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_comment_ws_message, LemmyContext, UserOperationCrud};

impl CreateOrUpdateComment {
  pub async fn send(
    comment: &ApubComment,
    actor: &ApubPerson,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    // TODO: might be helpful to add a comment method to retrieve community directly
    let post_id = comment.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    let community_id = post.community_id;
    let community: ApubCommunity = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();

    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let maa = collect_non_local_mentions(comment, &community, context).await?;

    let create_or_update = CreateOrUpdateComment {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: comment.to_apub(context).await?,
      cc: maa.ccs,
      tag: maa.tags,
      kind,
      id: id.clone(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::CreateOrUpdateComment(create_or_update);
    send_to_community(activity, &id, actor, &community, maa.inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdateComment {
  type DataType = LemmyContext;

  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to)?;
    let post = self.object.get_parents(context, request_counter).await?.0;
    let community = self.get_community(context, request_counter).await?;

    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    check_community_deleted_or_removed(&community)?;
    check_post_deleted_or_removed(&post)?;

    // TODO: should add a check that the correct community is in cc (probably needs changes to
    //       comment deserialization)
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let comment =
      ApubComment::from_apub(&self.object, context, self.actor.inner(), request_counter).await?;
    let recipients = get_notif_recipients(&self.actor, &comment, context, request_counter).await?;
    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreateComment,
      CreateOrUpdateType::Update => UserOperationCrud::EditComment,
    };
    send_comment_ws_message(
      comment.id, notif_type, None, None, None, recipients, context,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for CreateOrUpdateComment {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let post = self.object.get_parents(context, request_counter).await?.0;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, post.community_id)
    })
    .await??;
    Ok(community.into())
  }
}
