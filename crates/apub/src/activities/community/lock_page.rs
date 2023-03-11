use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_activity,
  local_instance,
  protocol::{
    activities::{
      community::lock_page::{LockPage, LockType, UndoLockPage},
      create_or_update::page::CreateOrUpdatePage,
      CreateOrUpdateType,
    },
    InCommunity,
  },
  SendActivity,
};
use activitypub_federation::{
  config::RequestData,
  fetch::object_id::ObjectId,
  kinds::{activity::UndoType, public},
  traits::ActivityHandler,
};
use lemmy_api_common::{
  context::LemmyContext,
  post::{LockPost, PostResponse},
  utils::get_local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::{
    community::Community,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait]
impl ActivityHandler for LockPage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &RequestData<Self::DataType>) -> Result<(), Self::Error> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_community_deleted_or_removed(&community)?;
    verify_mod_action(&self.actor, self.object.inner(), community.id, context).await?;
    Ok(())
  }

  async fn receive(self, context: &RequestData<Self::DataType>) -> Result<(), Self::Error> {
    let form = PostUpdateForm::builder().locked(Some(true)).build();
    let post = self.object.dereference(context).await?;
    Post::update(context.pool(), post.id, &form).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoLockPage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &RequestData<Self::DataType>) -> Result<(), Self::Error> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_community_deleted_or_removed(&community)?;
    verify_mod_action(
      &self.actor,
      self.object.object.inner(),
      community.id,
      context,
    )
    .await?;
    Ok(())
  }

  async fn receive(self, context: &RequestData<Self::DataType>) -> Result<(), Self::Error> {
    insert_activity(&self.id, &self, false, false, context).await?;
    let form = PostUpdateForm::builder().locked(Some(false)).build();
    let post = self.object.object.dereference(context).await?;
    Post::update(context.pool(), post.id, &form).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl SendActivity for LockPost {
  type Response = PostResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&request.auth, context.pool(), context.secret()).await?;
    // For backwards compat with 0.17
    CreateOrUpdatePage::send(
      &response.post_view.post,
      local_user_view.person.id,
      CreateOrUpdateType::Update,
      context,
    )
    .await?;
    let id = generate_activity_id(
      LockType::Lock,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let community_id = response.post_view.community.actor_id.clone();
    let actor = local_user_view.person.actor_id.clone().into();
    let lock = LockPage {
      actor,
      to: vec![public()],
      object: response.post_view.post.ap_id.clone().into(),
      cc: vec![community_id.clone().into()],
      kind: LockType::Lock,
      id,
      audience: Some(community_id.into()),
    };
    let activity = if request.locked {
      AnnouncableActivities::LockPost(lock)
    } else {
      let id = generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?;
      let undo = UndoLockPage {
        actor: lock.actor.clone(),
        to: vec![public()],
        cc: lock.cc.clone(),
        kind: UndoType::Undo,
        id,
        audience: lock.audience.clone(),
        object: lock,
      };
      AnnouncableActivities::UndoLockPost(undo)
    };
    let community = Community::read(context.pool(), response.post_view.community.id).await?;
    send_activity_in_community(
      activity,
      &local_user_view.person.into(),
      &community.into(),
      vec![],
      true,
      context,
    )
    .await?;
    Ok(())
  }
}
