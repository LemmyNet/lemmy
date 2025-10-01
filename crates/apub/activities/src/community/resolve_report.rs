use super::{local_community, report_inboxes, verify_mod_or_admin_action};
use crate::{
  activity_lists::AnnouncableActivities,
  generate_activity_id,
  protocol::community::{
    announce::AnnounceActivity,
    report::Report,
    resolve_report::{ResolveReport, ResolveType},
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::verification::verify_urls_match,
  traits::{Activity, Object},
};
use either::Either;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{
    community::ApubCommunity,
    instance::ApubSite,
    person::ApubPerson,
    PostOrComment,
    ReportableObjects,
  },
  utils::functions::verify_person_in_site_or_community,
};
use lemmy_db_schema::{
  source::{
    comment_report::CommentReport,
    community_report::CommunityReport,
    post_report::PostReport,
  },
  traits::Reportable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl ResolveReport {
  pub(crate) async fn send(
    object_id: ObjectId<ReportableObjects>,
    actor: &ApubPerson,
    report_creator: &ApubPerson,
    receiver: &Either<ApubSite, ApubCommunity>,
    context: Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let kind = ResolveType::Resolve;
    let id = generate_activity_id(kind.clone(), &context)?;
    let object = Report::new(&object_id, report_creator, receiver, None, &context)?;
    let resolve = ResolveReport {
      actor: actor.id().clone().into(),
      to: [receiver.id().clone().into()],
      object,
      kind,
      id: id.clone(),
    };
    let inboxes = report_inboxes(object_id, receiver, report_creator, &context).await?;

    send_lemmy_activity(&context, resolve, actor, inboxes, false).await
  }
}

#[async_trait::async_trait]
impl Activity for ResolveReport {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    self.object.verify(context).await?;
    let receiver = self.object.to[0].dereference(context).await?;
    verify_person_in_site_or_community(&self.actor, &receiver, context).await?;
    verify_urls_match(self.to[0].inner(), self.object.to[0].inner())?;
    verify_mod_or_admin_action(&self.actor, &receiver, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let reporter = self.object.actor.dereference(context).await?;
    let actor = self.actor.dereference(context).await?;
    match self.object.object.dereference(context).await? {
      ReportableObjects::Left(PostOrComment::Left(post)) => {
        PostReport::resolve_apub(&mut context.pool(), post.id, reporter.id, actor.id).await?;
      }
      ReportableObjects::Left(PostOrComment::Right(comment)) => {
        CommentReport::resolve_apub(&mut context.pool(), comment.id, reporter.id, actor.id).await?;
      }
      ReportableObjects::Right(community) => {
        CommunityReport::resolve_apub(&mut context.pool(), community.id, reporter.id, actor.id)
          .await?;
      }
    };

    let receiver = self.object.to[0].dereference(context).await?;
    if let Some(community) = local_community(&receiver) {
      // forward to remote mods
      let object_id = self.object.object.object_id(context).await?;
      let announce = AnnouncableActivities::ResolveReport(self);
      let announce = AnnounceActivity::new(announce.try_into()?, community, context)?;
      let inboxes = report_inboxes(object_id, &receiver, &reporter, context).await?;
      send_lemmy_activity(context, announce, community, inboxes.clone(), false).await?;
    }

    Ok(())
  }
}
