use crate::{
  activities::{
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{activities::community::collection_remove::CollectionRemove, InCommunity},
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor},
};
use activitystreams_kinds::{activity::RemoveType, public};
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
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      RemoveType::Remove,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let remove = CollectionRemove {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: ObjectId::new(removed_mod.actor_id()),
      target: generate_moderators_url(&community.actor_id)?.into(),
      id: id.clone(),
      cc: vec![community.actor_id()],
      kind: RemoveType::Remove,
      audience: Some(ObjectId::new(community.actor_id())),
    };

    let activity = AnnouncableActivities::CollectionRemove(remove);
    let inboxes = vec![removed_mod.shared_inbox_or_inbox()];
    send_activity_in_community(activity, actor, community, inboxes, true, context).await
  }

  pub async fn send_remove_featured_post(
    community: &ApubCommunity,
    featured_post: &ApubPost,
    actor: &ApubPerson,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      RemoveType::Remove,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let remove = CollectionRemove {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: featured_post.ap_id.clone().into(),
      target: generate_featured_url(&community.actor_id)?.into(),
      cc: vec![community.actor_id()],
      kind: RemoveType::Remove,
      id: id.clone(),
      audience: Some(ObjectId::new(community.actor_id())),
    };
    let activity = AnnouncableActivities::CollectionRemove(remove);
    send_activity_in_community(activity, actor, community, vec![], true, context).await
  }
}

#[async_trait::async_trait(?Send)]
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
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_mod_action(
      &self.actor,
      self.object.inner(),
      community.id,
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let (community, collection_type) =
      Community::get_by_collection_url(context.pool(), &self.target.into()).await?;
    match collection_type {
      CollectionType::Moderators => {
        let remove_mod = self
          .object
          .dereference(context, local_instance(context).await, request_counter)
          .await?;

        let form = CommunityModeratorForm {
          community_id: community.id,
          person_id: remove_mod.id,
        };
        CommunityModerator::leave(context.pool(), &form).await?;

        // write mod log
        let actor = self
          .actor
          .dereference(context, local_instance(context).await, request_counter)
          .await?;
        let form = ModAddCommunityForm {
          mod_person_id: actor.id,
          other_person_id: remove_mod.id,
          community_id: community.id,
          removed: Some(true),
        };
        ModAddCommunity::create(context.pool(), &form).await?;

        // TODO: send websocket notification about removed mod
      }
      CollectionType::Featured => {
        let post = ObjectId::<ApubPost>::new(self.object)
          .dereference(context, local_instance(context).await, request_counter)
          .await?;
        let form = PostUpdateForm::builder()
          .featured_community(Some(false))
          .build();
        Post::update(context.pool(), post.id, &form).await?;
      }
    }
    Ok(())
  }
}
