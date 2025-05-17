use super::report_inboxes;
use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_mod_action},
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  protocol::activities::community::{
    announce::AnnounceActivity,
    report::Report,
    resolve_report::{ResolveReport, ResolveType},
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson, PostOrComment},
  utils::{functions::verify_person_in_community, protocol::InCommunity},
};
use lemmy_db_schema::{
  source::{comment_report::CommentReport, post_report::PostReport},
  traits::Reportable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl ResolveReport {
  pub(crate) async fn send(
    object_id: ObjectId<PostOrComment>,
    actor: &ApubPerson,
    report_creator: &ApubPerson,
    community: &ApubCommunity,
    context: Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let kind = ResolveType::Resolve;
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let object = Report::new(&object_id, report_creator, community, None, &context)?;
    let resolve = ResolveReport {
      actor: actor.id().into(),
      to: [community.id().into()],
      object,
      kind,
      id: id.clone(),
    };
    let inboxes = report_inboxes(object_id, community, &context).await?;

    send_lemmy_activity(&context, resolve, actor, inboxes, false).await
  }
}

#[async_trait::async_trait]
impl ActivityHandler for ResolveReport {
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
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_urls_match(self.to[0].inner(), self.object.to[0].inner())?;
    verify_mod_action(&self.actor, &community, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let reporter = self.object.actor.dereference(context).await?;
    let actor = self.actor.dereference(context).await?;
    match self.object.object.dereference(context).await? {
      PostOrComment::Left(post) => {
        PostReport::resolve_apub(&mut context.pool(), post.id, reporter.id, actor.id).await?;
      }
      PostOrComment::Right(comment) => {
        CommentReport::resolve_apub(&mut context.pool(), comment.id, reporter.id, actor.id).await?;
      }
    };

    let community = self.community(context).await?;
    if community.local {
      // forward to remote mods
      let object_id = self.object.object.object_id(context).await?;
      let announce = AnnouncableActivities::ResolveReport(self);
      let announce = AnnounceActivity::new(announce.try_into()?, &community, context)?;
      let inboxes = report_inboxes(object_id, &community, context).await?;
      send_lemmy_activity(context, announce, &community, inboxes.clone(), false).await?;
    }

    Ok(())
  }
}
