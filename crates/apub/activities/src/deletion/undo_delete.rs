use crate::{
  deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
  generate_activity_id,
  protocol::deletion::{delete::Delete, undo_delete::UndoDelete},
};
use activitypub_federation::{config::Data, kinds::activity::UndoType, traits::Activity};
use lemmy_api_utils::{context::LemmyContext, notify::notify_mod_action};
use lemmy_apub_objects::objects::person::ApubPerson;
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    community::{Community, CommunityUpdateForm},
    modlog::{Modlog, ModlogInsertForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult, UntranslatedError};
use url::Url;

#[async_trait::async_trait]
impl Activity for UndoDelete {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
    self.object.verify(data).await?;
    verify_delete_activity(&self.object, self.object.summary.is_some(), data).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    if let Some(reason) = self.object.summary {
      UndoDelete::receive_undo_remove_action(
        &self.actor.dereference(context).await?,
        self.object.object.id(),
        reason,
        context,
      )
      .await
    } else {
      receive_delete_action(self.object.object.id(), &self.actor, false, None, context).await
    }
  }
}

impl UndoDelete {
  pub(in crate::deletion) fn new(
    actor: &ApubPerson,
    object: DeletableObjects,
    to: Vec<Url>,
    community: Option<&Community>,
    summary: Option<String>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<UndoDelete> {
    let object = Delete::new(actor, object, to.clone(), community, summary, context)?;

    let id = generate_activity_id(UndoType::Undo, context)?;
    let cc: Option<Url> = community.map(|c| c.ap_id.clone().into());
    Ok(UndoDelete {
      actor: actor.ap_id.clone().into(),
      to,
      object,
      cc: cc.into_iter().collect(),
      kind: UndoType::Undo,
      id,
    })
  }

  pub(crate) async fn receive_undo_remove_action(
    actor: &ApubPerson,
    object: &Url,
    reason: String,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    match DeletableObjects::read_from_db(object, context).await? {
      DeletableObjects::Community(community) => {
        if community.local {
          Err(UntranslatedError::OnlyLocalAdminCanRestoreCommunity)?
        }
        let community_owner =
          CommunityModeratorView::top_mod_for_community(&mut context.pool(), community.id).await?;
        let form = ModlogInsertForm::admin_remove_community(
          actor.id,
          community.id,
          community_owner,
          false,
          &reason,
        );
        let action = Modlog::create(&mut context.pool(), &[form]).await?;
        notify_mod_action(action.clone(), context.app_data());

        Community::update(
          &mut context.pool(),
          community.id,
          &CommunityUpdateForm {
            removed: Some(false),
            ..Default::default()
          },
        )
        .await?;
      }
      DeletableObjects::Post(post) => {
        let form = ModlogInsertForm::mod_remove_post(actor.id, &post, false, &reason);
        let action = Modlog::create(&mut context.pool(), &[form]).await?;
        notify_mod_action(action, context.app_data());
        Post::update(
          &mut context.pool(),
          post.id,
          &PostUpdateForm {
            removed: Some(false),
            ..Default::default()
          },
        )
        .await?;
      }
      DeletableObjects::Comment(comment) => {
        let form = ModlogInsertForm::mod_remove_comment(actor.id, &comment, false, &reason);
        let action = Modlog::create(&mut context.pool(), &[form]).await?;
        notify_mod_action(action, context.app_data());
        Comment::update(
          &mut context.pool(),
          comment.id,
          &CommentUpdateForm {
            removed: Some(false),
            ..Default::default()
          },
        )
        .await?;
      }
      // TODO these need to be implemented yet, for now, return errors
      DeletableObjects::PrivateMessage(_) => Err(LemmyErrorType::NotFound)?,
      DeletableObjects::Person(_) => Err(LemmyErrorType::NotFound)?,
    }
    Ok(())
  }
}
