use crate::{
  activities::{
    post::send_websocket_message,
    verify_activity,
    verify_mod_action,
    verify_person_in_community,
  },
  objects::{FromApub, FromApubToForm},
  ActorType,
  PageExt,
};
use activitystreams::{activity::kind::UpdateType, base::BaseExt};
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match_opt, ActivityCommonFields, ActivityHandler, PublicUrl};
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::{
  source::post::{Post, PostForm},
  DbUrl,
};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePost {
  to: PublicUrl,
  object: PageExt,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: UpdateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UpdatePost {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    let community =
      verify_person_in_community(&self.common.actor, &self.cc, context, request_counter).await?;

    let temp_post = PostForm::from_apub(
      &self.object,
      context,
      self.common.actor.clone(),
      request_counter,
      true,
    )
    .await?;
    let post_id: DbUrl = temp_post.ap_id.context(location_info!())?;
    let old_post = blocking(context.pool(), move |conn| {
      Post::read_from_apub_id(conn, &post_id)
    })
    .await??;
    let stickied = temp_post.stickied.context(location_info!())?;
    let locked = temp_post.locked.context(location_info!())?;
    // community mod changed locked/sticky status
    if (stickied != old_post.stickied) || (locked != old_post.locked) {
      verify_mod_action(&self.common.actor, community.actor_id(), context).await?;
    }
    // user edited their own post
    else {
      verify_domains_match_opt(&self.common.actor, self.object.id_unchecked())?;
    }

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
