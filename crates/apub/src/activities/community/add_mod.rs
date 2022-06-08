use crate::{
  activities::{
    community::{
      announce::GetCommunity,
      get_community_from_moderators_url,
      send_activity_in_community,
    },
    generate_activity_id,
    verify_add_remove_moderator_target,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  generate_moderators_url,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::community::add_mod::AddMod,
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor},
};
use activitystreams_kinds::{activity::AddType, public};
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::{
    community::{CommunityModerator, CommunityModeratorForm},
    moderator::{ModAddCommunity, ModAddCommunityForm},
  },
  traits::{Crud, Joinable},
};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

impl AddMod {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    community: &ApubCommunity,
    added_mod: &ApubPerson,
    actor: &ApubPerson,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      AddType::Add,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let add = AddMod {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: ObjectId::new(added_mod.actor_id()),
      target: generate_moderators_url(&community.actor_id)?.into(),
      cc: vec![community.actor_id()],
      kind: AddType::Add,
      id: id.clone(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::AddMod(add);
    let inboxes = vec![added_mod.shared_inbox_or_inbox()];
    send_activity_in_community(activity, actor, community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for AddMod {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_mod_action(
      &self.actor,
      self.object.inner(),
      &community,
      context,
      request_counter,
    )
    .await?;
    verify_add_remove_moderator_target(&self.target, &community)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.get_community(context, request_counter).await?;
    let new_mod = self
      .object
      .dereference(context, local_instance(context), request_counter)
      .await?;

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

      // write mod log
      let actor = self
        .actor
        .dereference(context, local_instance(context), request_counter)
        .await?;
      let form = ModAddCommunityForm {
        mod_person_id: actor.id,
        other_person_id: new_mod.id,
        community_id: community.id,
        removed: Some(false),
      };
      blocking(context.pool(), move |conn| {
        ModAddCommunity::create(conn, &form)
      })
      .await??;
    }
    // TODO: send websocket notification about added mod
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for AddMod {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    get_community_from_moderators_url(&self.target, context, request_counter).await
  }
}
