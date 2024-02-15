use crate::{
  activities::{
    deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
    generate_activity_id,
  },
  insert_received_activity,
  objects::person::ApubPerson,
  protocol::{activities::deletion::delete::Delete, IdOrNestedObject},
};
use activitypub_federation::{config::Data, kinds::activity::DeleteType, traits::ActivityHandler};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    comment_report::CommentReport,
    community::{Community, CommunityUpdateForm},
    moderator::{
      ModRemoveComment,
      ModRemoveCommentForm,
      ModRemoveCommunity,
      ModRemoveCommunityForm,
      ModRemovePost,
      ModRemovePostForm,
    },
    post::{Post, PostUpdateForm},
    post_report::PostReport,
  },
  traits::{Crud, Reportable},
};
use lemmy_utils::error::{LemmyError, LemmyErrorType};
use url::Url;

#[async_trait::async_trait]
impl ActivityHandler for Delete {
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
    verify_delete_activity(self, self.summary.is_some(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    if let Some(reason) = self.summary {
      // We set reason to empty string if it doesn't exist, to distinguish between delete and
      // remove. Here we change it back to option, so we don't write it to db.
      let reason = if reason.is_empty() {
        None
      } else {
        Some(reason)
      };
      receive_remove_action(
        &self.actor.dereference(context).await?,
        self.object.id(),
        reason,
        context,
      )
      .await
    } else {
      receive_delete_action(
        self.object.id(),
        &self.actor,
        true,
        self.remove_data,
        context,
      )
      .await
    }
  }
}

impl Delete {
  pub(in crate::activities::deletion) fn new(
    actor: &ApubPerson,
    object: DeletableObjects,
    to: Url,
    community: Option<&Community>,
    summary: Option<String>,
    context: &Data<LemmyContext>,
  ) -> Result<Delete, LemmyError> {
    let id = generate_activity_id(
      DeleteType::Delete,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let cc: Option<Url> = community.map(|c| c.actor_id.clone().into());
    Ok(Delete {
      actor: actor.actor_id.clone().into(),
      to: vec![to],
      object: IdOrNestedObject::Id(object.id()),
      cc: cc.into_iter().collect(),
      kind: DeleteType::Delete,
      summary,
      id,
      audience: community.map(|c| c.actor_id.clone().into()),
      remove_data: None,
    })
  }
}

#[tracing::instrument(skip_all)]
pub(in crate::activities) async fn receive_remove_action(
  actor: &ApubPerson,
  object: &Url,
  reason: Option<String>,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  match DeletableObjects::read_from_db(object, context).await? {
    DeletableObjects::Community(community) => {
      if community.local {
        Err(LemmyErrorType::OnlyLocalAdminCanRemoveCommunity)?
      }
      let form = ModRemoveCommunityForm {
        mod_person_id: actor.id,
        community_id: community.id,
        removed: Some(true),
        reason,
      };
      ModRemoveCommunity::create(&mut context.pool(), &form).await?;
      Community::update(
        &mut context.pool(),
        community.id,
        &CommunityUpdateForm {
          removed: Some(true),
          ..Default::default()
        },
      )
      .await?;
    }
    DeletableObjects::Post(post) => {
      PostReport::resolve_all_for_object(&mut context.pool(), post.id, actor.id).await?;
      let form = ModRemovePostForm {
        mod_person_id: actor.id,
        post_id: post.id,
        removed: Some(true),
        reason,
      };
      ModRemovePost::create(&mut context.pool(), &form).await?;
      Post::update(
        &mut context.pool(),
        post.id,
        &PostUpdateForm {
          removed: Some(true),
          ..Default::default()
        },
      )
      .await?;
    }
    DeletableObjects::Comment(comment) => {
      CommentReport::resolve_all_for_object(&mut context.pool(), comment.id, actor.id).await?;
      let form = ModRemoveCommentForm {
        mod_person_id: actor.id,
        comment_id: comment.id,
        removed: Some(true),
        reason,
      };
      ModRemoveComment::create(&mut context.pool(), &form).await?;
      Comment::update(
        &mut context.pool(),
        comment.id,
        &CommentUpdateForm {
          removed: Some(true),
          ..Default::default()
        },
      )
      .await?;
    }
    DeletableObjects::PrivateMessage(_) => unimplemented!(),
    DeletableObjects::Person { .. } => unimplemented!(),
  }
  Ok(())
}
