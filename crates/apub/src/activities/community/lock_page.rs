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
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::{activity::UndoType, public};
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

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LockPage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(
    &self,
    context: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    check_community_deleted_or_removed(&community)?;
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

  async fn receive(
    self,
    context: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    let form = PostUpdateForm::builder().locked(Some(true)).build();
    let post = self
      .object
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    Post::update(context.pool(), post.id, &form).await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoLockPage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(
    &self,
    context: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    check_community_deleted_or_removed(&community)?;
    verify_mod_action(
      &self.actor,
      self.object.object.inner(),
      community.id,
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    let form = PostUpdateForm::builder().locked(Some(false)).build();
    let post = self
      .object
      .object
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    Post::update(context.pool(), post.id, &form).await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
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
    let community_id: Url = response.post_view.community.actor_id.clone().into();
    let actor = ObjectId::new(local_user_view.person.actor_id.clone());
    let lock = LockPage {
      actor,
      to: vec![public()],
      object: ObjectId::new(response.post_view.post.ap_id.clone()),
      cc: vec![community_id.clone()],
      kind: LockType::Lock,
      id,
      audience: Some(ObjectId::new(community_id)),
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
