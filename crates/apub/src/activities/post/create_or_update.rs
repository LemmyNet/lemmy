use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    extract_community,
    generate_activity_id,
    verify_activity,
    verify_mod_action,
    verify_person_in_community,
    CreateOrUpdateType,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  objects::{post::Page, FromApub, ToApub},
  ActorType,
};
use activitystreams::{base::AnyBase, primitives::OneOrMany, unparsed::Unparsed};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  values::PublicUrl,
  verify_domains_match,
  verify_urls_match,
  ActivityFields,
  ActivityHandler,
};
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{community::Community, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePost {
  actor: Url,
  to: PublicUrl,
  object: Page,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: CreateOrUpdateType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl CreateOrUpdatePost {
  pub async fn send(
    post: &Post,
    actor: &Person,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let id = generate_activity_id(kind.clone())?;
    let create_or_update = CreateOrUpdatePost {
      actor: actor.actor_id(),
      to: PublicUrl::Public,
      object: post.to_apub(context.pool()).await?,
      cc: [community.actor_id()],
      kind,
      id: id.clone(),
      context: lemmy_context(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::CreateOrUpdatePost(Box::new(create_or_update));
    send_to_community_new(activity, &id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdatePost {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    let community = extract_community(&self.cc, context, request_counter).await?;
    let community_id = community.actor_id();
    verify_person_in_community(&self.actor, &community_id, context, request_counter).await?;
    match self.kind {
      CreateOrUpdateType::Create => {
        verify_domains_match(&self.actor, self.object.id_unchecked())?;
        verify_urls_match(&self.actor, &self.object.attributed_to)?;
        // Check that the post isnt locked or stickied, as that isnt possible for newly created posts.
        // However, when fetching a remote post we generate a new create activity with the current
        // locked/stickied value, so this check may fail. So only check if its a local community,
        // because then we will definitely receive all create and update activities separately.
        let is_stickied_or_locked =
          self.object.stickied == Some(true) || self.object.comments_enabled == Some(false);
        if community.local && is_stickied_or_locked {
          return Err(anyhow!("New post cannot be stickied or locked").into());
        }
      }
      CreateOrUpdateType::Update => {
        let is_mod_action = self.object.is_mod_action(context.pool()).await?;
        if is_mod_action {
          verify_mod_action(&self.actor, community_id, context).await?;
        } else {
          verify_domains_match(&self.actor, self.object.id_unchecked())?;
          verify_urls_match(&self.actor, &self.object.attributed_to)?;
        }
      }
    }
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = get_or_fetch_and_upsert_person(&self.actor, context, request_counter).await?;
    let post = Post::from_apub(&self.object, context, &actor.actor_id(), request_counter).await?;

    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreatePost,
      CreateOrUpdateType::Update => UserOperationCrud::EditPost,
    };
    send_post_ws_message(post.id, notif_type, None, None, context).await?;
    Ok(())
  }
}
