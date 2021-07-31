use crate::{
  activities::{
    comment::{collect_non_local_mentions, get_notif_recipients, send_websocket_message},
    community::announce::AnnouncableActivities,
    extract_community,
    generate_activity_id,
    verify_activity,
    verify_person_in_community,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  objects::{comment::Note, FromApub, ToApub},
  ActorType,
};
use activitystreams::{activity::kind::UpdateType, link::Mention};
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
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComment {
  to: PublicUrl,
  object: Note,
  cc: Vec<Url>,
  tag: Vec<Mention>,
  #[serde(rename = "type")]
  kind: UpdateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

impl UpdateComment {
  pub async fn send(
    comment: &Comment,
    actor: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    // TODO: would be helpful to add a comment method to retrieve community directly
    let post_id = comment.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;
    let id = generate_activity_id(UpdateType::Update)?;
    let maa = collect_non_local_mentions(comment, &community, context).await?;

    let update = UpdateComment {
      to: PublicUrl::Public,
      object: comment.to_apub(context.pool()).await?,
      cc: maa.ccs,
      tag: maa.tags,
      kind: Default::default(),
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };

    let activity = AnnouncableActivities::UpdateComment(update);
    send_to_community_new(activity, &id, actor, &community, maa.inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UpdateComment {
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
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
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
    send_websocket_message(
      comment.id,
      recipients,
      UserOperationCrud::EditComment,
      context,
    )
    .await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
