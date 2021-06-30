use crate::activities::{
  community::verify_add_remove_moderator_target,
  verify_mod_action,
  LemmyActivity,
};
use activitystreams::{activity::kind::AddType, base::AnyBase};
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  CommunityType,
};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler, PublicUrl};
use lemmy_db_queries::{source::community::CommunityModerator_, Joinable};
use lemmy_db_schema::source::{
  community::{CommunityModerator, CommunityModeratorForm},
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMod {
  to: PublicUrl,
  object: Url,
  target: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: AddType,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<AddMod> {
  type Actor = Person;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    verify_domains_match(&self.inner.target, &self.inner.cc[0])?;
    check_is_apub_id_valid(&self.actor, false)?;
    verify_mod_action(self.actor.clone(), self.inner.cc[0].clone(), context).await?;
    verify_add_remove_moderator_target(&self.inner.target, self.inner.cc[0].clone())
  }

  async fn receive(
    &self,
    _actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.inner.cc[0], context, request_counter).await?;
    let new_mod =
      get_or_fetch_and_upsert_person(&self.inner.object, context, request_counter).await?;

    // If we had to refetch the community while parsing the activity, then the new mod has already
    // been added. Skip it here as it would result in a duplicate key error.
    let new_mod_id = new_mod.id;
    let moderated_communities = blocking(context.pool(), move |conn| {
      CommunityModerator::get_person_moderated_communities(conn, new_mod_id)
    })
    .await??;
    if !moderated_communities.contains(&community.id) {
      let form = CommunityModeratorForm {
        community_id: community.id,
        person_id: new_mod.id,
      };
      blocking(context.pool(), move |conn| {
        CommunityModerator::join(conn, &form)
      })
      .await??;
    }
    if community.local {
      let anybase = AnyBase::from_arbitrary_json(serde_json::to_string(self)?)?;
      community
        .send_announce(anybase, Some(self.inner.object.clone()), context)
        .await?;
    }
    // TODO: send websocket notification about added mod
    Ok(())
  }
}
