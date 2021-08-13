use crate::{
  activities::{verify_activity, verify_mod_action, verify_person_in_community},
  objects::community::Group,
};
use activitystreams::activity::kind::UpdateType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::{ApubObject, Crud};
use lemmy_db_schema::source::community::{Community, CommunityForm};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};
use url::Url;

/// This activity is received from a remote community mod, and updates the description or other
/// fields of a local community.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommunity {
  to: PublicUrl,
  object: Group,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UpdateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UpdateCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &self.cc[0], context, request_counter).await?;
    verify_mod_action(&self.common.actor, self.cc[0].clone(), context).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let cc = self.cc[0].clone().into();
    let community = blocking(context.pool(), move |conn| {
      Community::read_from_apub_id(conn, &cc)
    })
    .await??;

    let updated_community =
      Group::from_apub_to_form(&self.object, &community.actor_id.clone().into()).await?;
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

    send_community_ws_message(
      updated_community.id,
      UserOperationCrud::EditCommunity,
      None,
      None,
      context,
    )
    .await?;
    Ok(())
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
