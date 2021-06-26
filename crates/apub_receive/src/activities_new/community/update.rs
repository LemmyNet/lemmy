use crate::{
  activities_new::community::{send_websocket_message, verify_is_community_mod},
  inbox::new_inbox_routing::Activity,
};
use activitystreams::{activity::kind::UpdateType, base::BaseExt};
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, objects::FromApubToForm, GroupExt};
use lemmy_apub_lib::{PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::{ApubObject, Crud};
use lemmy_db_schema::source::community::{Community, CommunityForm};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

/// This activity is received from a remote community mod, and updates the description or other
/// fields of a local community.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommunity {
  actor: Url,
  to: PublicUrl,
  object: GroupExt,
  cc: Url,
  #[serde(rename = "type")]
  kind: UpdateType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<UpdateCommunity> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    self.inner.object.id(self.inner.cc.as_str())?;
    check_is_apub_id_valid(&self.inner.actor, false)?;
    verify_is_community_mod(self.inner.actor.clone(), self.inner.cc.clone(), context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UpdateCommunity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let cc = self.inner.cc.clone().into();
    let community = blocking(context.pool(), move |conn| {
      Community::read_from_apub_id(conn, &cc)
    })
    .await??;

    let updated_community = CommunityForm::from_apub(
      &self.inner.object,
      context,
      community.actor_id.clone().into(),
      request_counter,
      false,
    )
    .await?;
    let cf = CommunityForm {
      name: updated_community.name,
      title: updated_community.title,
      description: updated_community.description,
      nsfw: updated_community.nsfw,
      // TODO: icon and banner would be hosted on the other instance, ideally we would copy it to ours
      icon: updated_community.icon,
      banner: updated_community.banner,
      ..CommunityForm::default()
    };
    let updated_community = blocking(context.pool(), move |conn| {
      Community::update(conn, community.id, &cf)
    })
    .await??;

    send_websocket_message(
      updated_community.id,
      UserOperationCrud::EditCommunity,
      context,
    )
    .await
  }
}
