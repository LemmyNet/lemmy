use crate::{
  activities::{
    deletion::{delete::receive_remove_action, verify_delete_activity},
    verify_activity,
    verify_add_remove_moderator_target,
    verify_mod_action,
    verify_person_in_community,
  },
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  CommunityType,
};
use activitystreams::{activity::kind::RemoveType, base::AnyBase};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::Joinable;
use lemmy_db_schema::source::community::{CommunityModerator, CommunityModeratorForm};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMod {
  to: PublicUrl,
  pub(in crate::activities::removal) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: RemoveType,
  // if target is set, this is means remove mod from community
  pub(in crate::activities::removal) target: Option<Url>,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for RemoveMod {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    if let Some(target) = &self.target {
      verify_person_in_community(&self.common.actor, &self.cc[0], context, request_counter).await?;
      verify_mod_action(&self.common.actor, self.cc[0].clone(), context).await?;
      verify_add_remove_moderator_target(target, self.cc[0].clone())?;
    } else {
      verify_delete_activity(
        &self.object,
        &self.cc[0],
        self.common(),
        true,
        context,
        request_counter,
      )
      .await?;
    }
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if self.target.is_some() {
      let community =
        get_or_fetch_and_upsert_community(&self.cc[0], context, request_counter).await?;
      let remove_mod =
        get_or_fetch_and_upsert_person(&self.object, context, request_counter).await?;

      let form = CommunityModeratorForm {
        community_id: community.id,
        person_id: remove_mod.id,
      };
      blocking(context.pool(), move |conn| {
        CommunityModerator::leave(conn, &form)
      })
      .await??;
      let anybase = AnyBase::from_arbitrary_json(serde_json::to_string(&self)?)?;
      community
        .send_announce(anybase, Some(self.object.clone()), context)
        .await?;
      // TODO: send websocket notification about removed mod
      Ok(())
    } else {
      receive_remove_action(
        &self.common.actor,
        &self.object,
        None,
        context,
        request_counter,
      )
      .await
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
