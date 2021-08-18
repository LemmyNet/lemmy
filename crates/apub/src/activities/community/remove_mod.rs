use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    deletion::{delete::receive_remove_action, verify_delete_activity},
    generate_activity_id,
    verify_activity,
    verify_add_remove_moderator_target,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  generate_moderators_url,
  ActorType,
};
use activitystreams::{
  activity::kind::RemoveType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityFields, ActivityHandler};
use lemmy_db_queries::Joinable;
use lemmy_db_schema::source::{
  community::{Community, CommunityModerator, CommunityModeratorForm},
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMod {
  actor: Url,
  to: PublicUrl,
  pub(in crate::activities) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: RemoveType,
  // if target is set, this is means remove mod from community
  pub(in crate::activities) target: Option<Url>,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl RemoveMod {
  pub async fn send(
    community: &Community,
    removed_mod: &Person,
    actor: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(RemoveType::Remove)?;
    let remove = RemoveMod {
      actor: actor.actor_id(),
      to: PublicUrl::Public,
      object: removed_mod.actor_id(),
      target: Some(generate_moderators_url(&community.actor_id)?.into()),
      id: id.clone(),
      context: lemmy_context(),
      cc: [community.actor_id()],
      kind: RemoveType::Remove,
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::RemoveMod(remove);
    let inboxes = vec![removed_mod.get_shared_inbox_or_inbox_url()];
    send_to_community_new(activity, &id, actor, community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for RemoveMod {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    if let Some(target) = &self.target {
      verify_person_in_community(&self.actor, &self.cc[0], context, request_counter).await?;
      verify_mod_action(&self.actor, self.cc[0].clone(), context).await?;
      verify_add_remove_moderator_target(target, self.cc[0].clone())?;
    } else {
      verify_delete_activity(
        &self.object,
        self,
        &self.cc[0],
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
      // TODO: send websocket notification about removed mod
      Ok(())
    } else {
      receive_remove_action(&self.actor, &self.object, None, context, request_counter).await
    }
  }
}
