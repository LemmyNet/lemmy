use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    extract_community,
    generate_activity_id,
    post::send_websocket_message,
    verify_activity,
    verify_person_in_community,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  objects::{post::Page, FromApub, ToApub},
  ActorType,
};
use activitystreams::activity::kind::CreateType;
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  values::PublicUrl,
  verify_domains_match,
  verify_urls_match,
  ActivityCommonFields,
  ActivityHandler,
};
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{community::Community, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePost {
  to: PublicUrl,
  object: Page,
  cc: [Url; 1],
  r#type: CreateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

impl CreatePost {
  pub async fn send(post: &Post, actor: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let id = generate_activity_id(CreateType::Create)?;
    let create = CreatePost {
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

    let activity = AnnouncableActivities::CreatePost(create);
    send_to_community_new(activity, &id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreatePost {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = extract_community(&self.cc, context, request_counter).await?;
    let community_id = &community.actor_id();

    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, community_id, context, request_counter).await?;
    verify_domains_match(&self.common.actor, &self.object.id)?;
    verify_urls_match(&self.common.actor, &self.object.attributed_to)?;
    // Check that the post isnt locked or stickied, as that isnt possible for newly created posts.
    // However, when fetching a remote post we generate a new create activity with the current
    // locked/stickied value, so this check may fail. So only check if its a local community,
    // because then we will definitely receive all create and update activities separately.
    let is_stickied_or_locked =
      self.object.stickied == Some(true) || self.object.comments_enabled == Some(false);
    if community.local && is_stickied_or_locked {
      return Err(anyhow!("New post cannot be stickied or locked").into());
    }
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor =
      get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
    let post = Post::from_apub(
      &self.object,
      context,
      actor.actor_id(),
      request_counter,
      false,
    )
    .await?;

    send_websocket_message(post.id, UserOperationCrud::CreatePost, context).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
