use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    generate_activity_id,
    post::send_websocket_message,
    verify_activity,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::community::get_or_fetch_and_upsert_community,
  objects::{post::Page, FromApub, ToApub},
  ActorType,
};
use activitystreams::activity::kind::UpdateType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, verify_urls_match, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{community::Community, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePost {
  to: PublicUrl,
  object: Page,
  cc: [Url; 1],
  r#type: UpdateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

impl UpdatePost {
  pub async fn send(post: &Post, actor: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let id = generate_activity_id(UpdateType::Update)?;
    let update = UpdatePost {
      to: PublicUrl::Public,
      object: post.to_apub(context.pool()).await?,
      cc: [community.actor_id()],
      r#type: Default::default(),
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };
    let activity = AnnouncableActivities::UpdatePost(update);
    send_to_community_new(activity, &id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UpdatePost {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community_id = get_or_fetch_and_upsert_community(&self.cc[0], context, request_counter)
      .await?
      .actor_id();
    let is_mod_action = self.object.is_mod_action(context.pool()).await?;

    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &community_id, context, request_counter).await?;
    if is_mod_action {
      verify_mod_action(&self.common.actor, community_id, context).await?;
    } else {
      verify_urls_match(&self.common.actor, &self.object.attributed_to)?;
    }
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let post = Post::from_apub(
      &self.object,
      context,
      self.common.actor.clone(),
      request_counter,
      // TODO: we already check here if the mod action is valid, can remove that check param
      true,
    )
    .await?;

    send_websocket_message(post.id, UserOperationCrud::EditPost, context).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
