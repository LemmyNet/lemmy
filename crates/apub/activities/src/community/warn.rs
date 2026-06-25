use crate::{
  check_community_deleted_or_removed,
  community::verify_mod_or_admin_action,
  generate_activity_id,
  protocol::community::warn::{Warn, WarnType},
  send_lemmy_activity,
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::Activity};
use either::Either;
use lemmy_api_utils::{context::LemmyContext, notify::notify_mod_action};
use lemmy_apub_objects::{
  objects::{PostOrComment, person::ApubPerson},
  utils::protocol::InCommunity,
};
use lemmy_db_schema::source::{
  activity::ActivitySendTargets,
  community::Community,
  modlog::{Modlog, ModlogInsertForm},
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_post::PostView;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl Warn {
  pub(crate) fn new(
    object_id: ObjectId<PostOrComment>,
    community: &Community,
    actor: &ApubPerson,
    recipient: ObjectId<ApubPerson>,
    reason: String,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    let kind = WarnType::Warn;
    let id = generate_activity_id(kind.clone(), context)?;
    Ok(Warn {
      actor: actor.ap_id.clone().into(),
      to: [recipient],
      object: object_id,
      summary: reason,
      kind,
      id,
      audience: community.clone().ap_id.into(),
    })
  }

  pub(crate) async fn send(
    post_or_comment: Either<PostView, CommentView>,
    reason: String,
    actor: ApubPerson,
    context: Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let recipient = post_or_comment.clone().either(|p| p.creator, |c| c.creator);
    let object_id = post_or_comment
      .clone()
      .either(|p| p.post.ap_id, |c| c.comment.ap_id);
    let community = post_or_comment.either(|p| p.community, |c| c.community);
    let warn = Self::new(
      object_id.into(),
      &community,
      &actor,
      recipient.ap_id.clone().into(),
      reason,
      &context,
    )?;
    let inbox = ActivitySendTargets::to_inbox(recipient.inbox_url.into());

    send_lemmy_activity(&context, warn, &actor, inbox, false).await
  }
}

#[async_trait::async_trait]
impl Activity for Warn {
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
    check_community_deleted_or_removed(&community)?;
    verify_mod_or_admin_action(
      &self.actor,
      self.object.inner(),
      &Either::Right(community),
      context,
    )
    .await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let actor = self.actor.dereference(context).await?;
    let community = self.audience.dereference(context).await?;
    let form = match self.object.dereference(context).await? {
      Either::Left(post) => {
        ModlogInsertForm::mod_create_post_warning(actor.id, &post, &self.summary)
      }
      Either::Right(comment) => ModlogInsertForm::mod_create_comment_warning(
        actor.id,
        &comment,
        community.id,
        &self.summary,
      ),
    };
    let action = Modlog::create(&mut context.pool(), &[form]).await?;
    notify_mod_action(action, context);
    Ok(())
  }
}
