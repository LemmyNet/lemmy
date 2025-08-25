use crate::{
  activities::{
    community::send_activity_in_community,
    determine_collection_type_from_target,
    generate_activity_id,
  },
  activity_lists::AnnouncableActivities,
  protocol::activities::community::collection_remove::CollectionRemove,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::RemoveType,
  traits::{Activity, Actor, Object},
};
use itertools::Itertools;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{generate_featured_url, generate_moderators_url},
};
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  utils::{
    functions::{
      community_from_objects,
      generate_to,
      verify_mod_action,
      verify_person_in_community,
      verify_visibility,
    },
    protocol::InCommunity,
  },
};
use lemmy_db_schema::{
  impls::community::CollectionType,
  source::{
    activity::ActivitySendTargets,
    community::{Community, CommunityActions, CommunityModeratorForm},
    mod_log::moderator::{ModAddToCommunity, ModAddToCommunityForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl CollectionRemove {
  pub(super) async fn send_remove_mod(
    community: &ApubCommunity,
    removed_mod: &ApubPerson,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let id = generate_activity_id(RemoveType::Remove, context)?;
    let remove = CollectionRemove {
      actor: actor.id().clone().into(),
      to: generate_to(community)?,
      object: removed_mod.id().clone(),
      target: generate_moderators_url(&community.ap_id)?.into(),
      id: id.clone(),
      cc: vec![community.id().clone()],
      kind: RemoveType::Remove,
    };

    let activity = AnnouncableActivities::CollectionRemove(remove);
    let inboxes = ActivitySendTargets::to_inbox(removed_mod.shared_inbox_or_inbox());
    send_activity_in_community(activity, actor, community, inboxes, true, context).await
  }

  pub(super) async fn send_remove_featured_post(
    community: &ApubCommunity,
    featured_post: &ApubPost,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let id = generate_activity_id(RemoveType::Remove, context)?;
    let remove = CollectionRemove {
      actor: actor.id().clone().into(),
      to: generate_to(community)?,
      object: featured_post.ap_id.clone().into(),
      target: generate_featured_url(&community.ap_id)?.into(),
      cc: vec![community.id().clone()],
      kind: RemoveType::Remove,
      id: id.clone(),
    };
    let activity = AnnouncableActivities::CollectionRemove(remove);
    send_activity_in_community(
      activity,
      actor,
      community,
      ActivitySendTargets::empty(),
      true,
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl Activity for CollectionRemove {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let community = self.community(context).await?;
    verify_visibility(&self.to, &self.cc, &community)?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_mod_action(&self.actor, &community, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let objects = self.to.iter().merge(self.cc.iter());
    let community = community_from_objects(objects, context).await?;

    let collection_type = if community.local {
      determine_collection_type_from_target(self.target.clone(), community.ap_id.clone())?
    } else {
      let (_, collection_type) =
        Community::get_by_collection_url(&mut context.pool(), &self.target.into()).await?;
      collection_type
    };

    match collection_type {
      CollectionType::Moderators => {
        let remove_mod = ObjectId::<ApubPerson>::from(self.object)
          .dereference(context)
          .await?;

        let form = CommunityModeratorForm::new(community.id, remove_mod.id);
        CommunityActions::leave(&mut context.pool(), &form).await?;

        // write mod log
        let actor = self.actor.dereference(context).await?;
        let form = ModAddToCommunityForm {
          mod_person_id: actor.id,
          other_person_id: remove_mod.id,
          community_id: community.id,
          removed: Some(true),
        };
        ModAddToCommunity::create(&mut context.pool(), &form).await?;

        // TODO: send websocket notification about removed mod
      }
      CollectionType::Featured => {
        let post = ObjectId::<ApubPost>::from(self.object)
          .dereference(context)
          .await?;
        let form = PostUpdateForm {
          featured_community: Some(false),
          ..Default::default()
        };
        Post::update(&mut context.pool(), post.id, &form).await?;
      }
    }
    Ok(())
  }
}
