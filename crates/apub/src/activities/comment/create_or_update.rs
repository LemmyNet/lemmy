use crate::{
  activities::{
    comment::{collect_non_local_mentions, get_notif_recipients, send_websocket_message},
    community::announce::AnnouncableActivities,
    extract_community,
    generate_activity_id,
    verify_activity,
    verify_person_in_community,
    CreateOrUpdateType,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  objects::{comment::Note, FromApub, ToApub},
  ActorType,
};
use activitystreams::link::Mention;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  values::PublicUrl,
  verify_domains_match,
  ActivityCommonFields,
  ActivityHandler,
};
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{comment::Comment, community::Community, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateComment {
  to: PublicUrl,
  object: Note,
  cc: Vec<Url>,
  tag: Vec<Mention>,
  #[serde(rename = "type")]
  kind: CreateOrUpdateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

impl CreateOrUpdateComment {
  pub async fn send(
    comment: &Comment,
    actor: &Person,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    // TODO: might be helpful to add a comment method to retrieve community directly
    let post_id = comment.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let id = generate_activity_id(kind.clone())?;
    let maa = collect_non_local_mentions(comment, &community, context).await?;

    let create_or_update = CreateOrUpdateComment {
      to: PublicUrl::Public,
      object: comment.to_apub(context.pool()).await?,
      cc: maa.ccs,
      tag: maa.tags,
      kind,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };

    let activity = AnnouncableActivities::CreateOrUpdateComment(create_or_update);
    send_to_community_new(activity, &id, actor, &community, maa.inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdateComment {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = extract_community(&self.cc, context, request_counter).await?;

    verify_activity(self.common())?;
    verify_person_in_community(
      &self.common.actor,
      &community.actor_id(),
      context,
      request_counter,
    )
    .await?;
    verify_domains_match(&self.common.actor, &self.object.id)?;
    // TODO: should add a check that the correct community is in cc (probably needs changes to
    //       comment deserialization)
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let comment = Comment::from_apub(
      &self.object,
      context,
      self.common.actor.clone(),
      request_counter,
      false,
    )
    .await?;
    let recipients =
      get_notif_recipients(&self.common.actor, &comment, context, request_counter).await?;
    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreateComment,
      CreateOrUpdateType::Update => UserOperationCrud::EditComment,
    };
    send_websocket_message(comment.id, recipients, notif_type, context).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
