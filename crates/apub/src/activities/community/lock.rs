use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::send_activity_in_community,
    generate_activity_id,
  },
  activity_lists::AnnouncableActivities,
  post_or_comment_community,
  protocol::activities::community::lock::{LockPageOrNote, LockType, UndoLockPageOrNote},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UndoType,
  traits::Activity,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, PostOrComment},
  utils::{
    functions::{generate_to, verify_mod_action, verify_person_in_community, verify_visibility},
    protocol::InCommunity,
  },
};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    comment::Comment,
    mod_log::moderator::{ModLockComment, ModLockCommentForm, ModLockPost, ModLockPostForm},
    person::Person,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[async_trait::async_trait]
impl Activity for LockPageOrNote {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let community = self.community(context).await?;
    verify_visibility(&self.to, &self.cc, &community)?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_community_deleted_or_removed(&community)?;
    verify_mod_action(&self.actor, &community, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let locked = true;
    let reason = self.summary;

    match self.object.dereference(context).await? {
      PostOrComment::Left(post) => {
        let form = PostUpdateForm {
          locked: Some(locked),
          ..Default::default()
        };
        Post::update(&mut context.pool(), post.id, &form).await?;

        let form = ModLockPostForm {
          mod_person_id: self.actor.dereference(context).await?.id,
          post_id: post.id,
          locked: Some(locked),
          reason,
        };
        ModLockPost::create(&mut context.pool(), &form).await?;
      }
      PostOrComment::Right(comment) => {
        Comment::update_locked_for_comment_and_children(&mut context.pool(), &comment.path, locked)
          .await?;

        let form = ModLockCommentForm {
          mod_person_id: self.actor.dereference(context).await?.id,
          comment_id: comment.id,
          locked: Some(locked),
          reason,
        };
        ModLockComment::create(&mut context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}

#[async_trait::async_trait]
impl Activity for UndoLockPageOrNote {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let community = self.object.community(context).await?;
    verify_visibility(&self.to, &self.cc, &community)?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_community_deleted_or_removed(&community)?;
    verify_mod_action(&self.actor, &community, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let locked = false;
    let reason = self.summary;

    match self.object.object.dereference(context).await? {
      PostOrComment::Left(post) => {
        let form = PostUpdateForm {
          locked: Some(locked),
          ..Default::default()
        };

        Post::update(&mut context.pool(), post.id, &form).await?;

        let form = ModLockPostForm {
          mod_person_id: self.actor.dereference(context).await?.id,
          post_id: post.id,
          locked: Some(locked),
          reason,
        };
        ModLockPost::create(&mut context.pool(), &form).await?;
      }
      PostOrComment::Right(comment) => {
        Comment::update_locked_for_comment_and_children(&mut context.pool(), &comment.path, locked)
          .await?;

        let form = ModLockCommentForm {
          mod_person_id: self.actor.dereference(context).await?.id,
          comment_id: comment.id,
          locked: Some(locked),
          reason,
        };
        ModLockComment::create(&mut context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}

pub(crate) async fn send_lock(
  object: PostOrComment,
  actor: Person,
  locked: bool,
  reason: Option<String>,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let community: ApubCommunity = post_or_comment_community(&object, &context).await?.into();
  let id = generate_activity_id(LockType::Lock, &context)?;
  let community_id = community.ap_id.inner().clone();
  let ap_id = match object {
    PostOrComment::Left(p) => p.ap_id.clone(),
    PostOrComment::Right(c) => c.ap_id.clone(),
  };

  let lock = LockPageOrNote {
    actor: actor.ap_id.clone().into(),
    to: generate_to(&community)?,
    object: ObjectId::from(ap_id),
    cc: vec![community_id.clone()],
    kind: LockType::Lock,
    id,
    summary: reason.clone(),
  };
  let activity = if locked {
    AnnouncableActivities::Lock(lock)
  } else {
    let id = generate_activity_id(UndoType::Undo, &context)?;
    let undo = UndoLockPageOrNote {
      actor: lock.actor.clone(),
      to: generate_to(&community)?,
      cc: lock.cc.clone(),
      kind: UndoType::Undo,
      id,
      object: lock,
      summary: reason,
    };
    AnnouncableActivities::UndoLock(undo)
  };
  send_activity_in_community(
    activity,
    &actor.into(),
    &community,
    ActivitySendTargets::empty(),
    true,
    &context,
  )
  .await?;
  Ok(())
}
