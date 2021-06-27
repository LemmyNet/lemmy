use crate::{activities_new::post::send_websocket_message, inbox::new_inbox_routing::Activity};
use activitystreams::{activity::kind::UpdateType, base::BaseExt};
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::person::get_or_fetch_and_upsert_person,
  objects::{FromApub, FromApubToForm},
  ActorType,
  PageExt,
};
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::{ApubObject, Crud};
use lemmy_db_schema::{
  source::{
    community::Community,
    post::{Post, PostForm},
  },
  DbUrl,
};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePost {
  actor: Url,
  to: PublicUrl,
  object: PageExt,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: UpdateType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<UpdatePost> {
  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(self.id_unchecked(), &self.inner.actor)?;
    self.inner.object.id(self.inner.actor.as_str())?;
    check_is_apub_id_valid(&self.inner.actor, false)
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UpdatePost> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let person =
      get_or_fetch_and_upsert_person(&self.inner.actor, context, request_counter).await?;
    let temp_post = PostForm::from_apub(
      &self.inner.object,
      context,
      person.actor_id(),
      request_counter,
      false,
    )
    .await?;

    let post_id: DbUrl = temp_post.ap_id.context(location_info!())?;
    let old_post = blocking(context.pool(), move |conn| {
      Post::read_from_apub_id(conn, &post_id)
    })
    .await??;

    // If sticked or locked state was changed, make sure the actor is a mod
    let stickied = temp_post.stickied.context(location_info!())?;
    let locked = temp_post.locked.context(location_info!())?;
    let mut mod_action_allowed = false;
    if (stickied != old_post.stickied) || (locked != old_post.locked) {
      let community = blocking(context.pool(), move |conn| {
        Community::read(conn, old_post.community_id)
      })
      .await??;
      // Only check mod status if the community is local, otherwise we trust that it was sent correctly.
      if community.local {
        // TODO
        //verify_mod_activity(&update, announce, &community, context).await?;
      }
      mod_action_allowed = true;
    }

    let post = Post::from_apub(
      &self.inner.object,
      context,
      person.actor_id(),
      request_counter,
      mod_action_allowed,
    )
    .await?;

    send_websocket_message(post.id, UserOperationCrud::EditPost, context).await
  }
}
