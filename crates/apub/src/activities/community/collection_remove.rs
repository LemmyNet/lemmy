use crate::{
  activities::{
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{activities::community::collection_remove::CollectionRemove, InCommunity},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::{activity::RemoveType, public},
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{generate_featured_url, generate_moderators_url},
};
use lemmy_db_schema::{
  impls::community::CollectionType,
  source::{
    community::{Community, CommunityModerator, CommunityModeratorForm},
    moderator::{ModAddCommunity, ModAddCommunityForm},
    post::{Post, PostUpdateForm},
  },
  traits::{Crud, Joinable},
};
use lemmy_utils::error::LemmyError;
use url::Url;

impl CollectionRemove {
  #[tracing::instrument(skip_all)]
  pub async fn send_remove_mod(
    community: &ApubCommunity,
    removed_mod: &ApubPerson,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      RemoveType::Remove,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let remove = CollectionRemove {
      actor: actor.id().into(),
      to: vec![public()],
      object: removed_mod.id(),
      target: generate_moderators_url(&community.actor_id)?.into(),
      id: id.clone(),
      cc: vec![community.id()],
      kind: RemoveType::Remove,
      audience: Some(community.id().into()),
    };

    let activity = AnnouncableActivities::CollectionRemove(remove);
    let inboxes = vec![removed_mod.shared_inbox_or_inbox()];
    send_activity_in_community(activity, actor, community, inboxes, true, context).await
  }

  pub async fn send_remove_featured_post(
    community: &ApubCommunity,
    featured_post: &ApubPost,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      RemoveType::Remove,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let remove = CollectionRemove {
      actor: actor.id().into(),
      to: vec![public()],
      object: featured_post.ap_id.clone().into(),
      target: generate_featured_url(&community.actor_id)?.into(),
      cc: vec![community.id()],
      kind: RemoveType::Remove,
      id: id.clone(),
      audience: Some(community.id().into()),
    };
    let activity = AnnouncableActivities::CollectionRemove(remove);
    send_activity_in_community(activity, actor, community, vec![], true, context).await
  }
}

#[async_trait::async_trait]
impl ActivityHandler for CollectionRemove {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_mod_action(&self.actor, &self.object, community.id, context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    let (community, collection_type) =
      Community::get_by_collection_url(&mut context.pool(), &self.target.into()).await?;
    match collection_type {
      CollectionType::Moderators => {
        let remove_mod = ObjectId::<ApubPerson>::from(self.object)
          .dereference(context)
          .await?;

        let form = CommunityModeratorForm {
          community_id: community.id,
          person_id: remove_mod.id,
        };
        CommunityModerator::leave(&mut context.pool(), &form).await?;

        // write mod log
        let actor = self.actor.dereference(context).await?;
        let form = ModAddCommunityForm {
          mod_person_id: actor.id,
          other_person_id: remove_mod.id,
          community_id: community.id,
          removed: Some(true),
        };
        ModAddCommunity::create(&mut context.pool(), &form).await?;

        // TODO: send websocket notification about removed mod
      }
      CollectionType::Featured => {
        let post = ObjectId::<ApubPost>::from(self.object)
          .dereference(context)
          .await?;
        let form = PostUpdateForm::builder()
          .featured_community(Some(false))
          .build();
        Post::update(&mut context.pool(), post.id, &form).await?;
      }
    }
    Ok(())
  }
}
