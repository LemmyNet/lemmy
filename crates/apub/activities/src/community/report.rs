use super::{local_community, report_inboxes};
use crate::{
  activity_lists::AnnouncableActivities,
  generate_activity_id,
  protocol::community::{
    announce::AnnounceActivity,
    report::{Report, ReportObject},
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::FlagType,
  traits::{Activity, Object},
};
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{
    check_comment_deleted_or_removed,
    check_community_deleted_removed,
    check_post_deleted_or_removed,
  },
};
use lemmy_apub_objects::{
  objects::{
    PostOrComment,
    ReportableObjects,
    community::ApubCommunity,
    instance::ApubSite,
    person::ApubPerson,
  },
  utils::functions::verify_person_in_site_or_community,
};
use lemmy_db_schema::{
  source::{
    comment_report::{CommentReport, CommentReportForm},
    community_report::{CommunityReport, CommunityReportForm},
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl Report {
  pub(crate) fn new(
    object_id: &ObjectId<ReportableObjects>,
    actor: &ApubPerson,
    receiver: &Either<ApubSite, ApubCommunity>,
    reason: Option<String>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    let kind = FlagType::Flag;
    let id = generate_activity_id(kind.clone(), context)?;
    Ok(Report {
      actor: actor.id().clone().into(),
      to: [receiver.id().clone().into()],
      object: ReportObject::Lemmy(object_id.clone()),
      summary: reason,
      content: None,
      kind,
      id: id.clone(),
      audience: receiver.as_ref().right().map(|c| c.ap_id.clone().into()),
    })
  }

  pub(crate) async fn send(
    object_id: ObjectId<ReportableObjects>,
    actor: &ApubPerson,
    receiver: &Either<ApubSite, ApubCommunity>,
    reason: String,
    context: Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let report = Self::new(&object_id, actor, receiver, Some(reason), &context)?;
    let inboxes = report_inboxes(object_id, receiver, actor, &context).await?;

    send_lemmy_activity(&context, report, actor, inboxes, false).await
  }
}

#[async_trait::async_trait]
impl Activity for Report {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let receiver = self.to[0].dereference(context).await?;
    verify_person_in_site_or_community(&self.actor, &receiver, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let actor = self.actor.dereference(context).await?;
    let reason = self.reason()?;
    match self.object.dereference(context).await? {
      ReportableObjects::Left(PostOrComment::Left(post)) => {
        check_post_deleted_or_removed(&post)?;

        let report_form = PostReportForm {
          creator_id: actor.id,
          post_id: post.id,
          original_post_name: post.name.clone(),
          original_post_url: post.url.clone(),
          reason,
          original_post_body: post.body.clone(),
          violates_instance_rules: false,
        };
        PostReport::report(&mut context.pool(), &report_form).await?;
      }
      ReportableObjects::Left(PostOrComment::Right(comment)) => {
        check_comment_deleted_or_removed(&comment)?;

        let report_form = CommentReportForm {
          creator_id: actor.id,
          comment_id: comment.id,
          original_comment_text: comment.content.clone(),
          reason,
          violates_instance_rules: false,
        };
        CommentReport::report(&mut context.pool(), &report_form).await?;
      }
      ReportableObjects::Right(community) => {
        check_community_deleted_removed(&community)?;
        let report_form = CommunityReportForm {
          creator_id: actor.id,
          community_id: community.id,
          reason,
          original_community_name: community.name.clone(),
          original_community_title: community.title.clone(),
          original_community_banner: community.banner.clone(),
          original_community_icon: community.icon.clone(),
          original_community_summary: community.summary.clone(),
          original_community_sidebar: community.sidebar.clone(),
        };
        CommunityReport::report(&mut context.pool(), &report_form).await?;
      }
    };

    let receiver = self.to[0].dereference(context).await?;
    if let Some(community) = local_community(&receiver) {
      // forward to remote mods
      let object_id = self.object.object_id(context).await?;
      let announce = AnnouncableActivities::Report(self);
      let announce = AnnounceActivity::new(announce.try_into()?, community, context)?;
      let inboxes = report_inboxes(object_id, &receiver, &actor, context).await?;
      send_lemmy_activity(context, announce, community, inboxes.clone(), false).await?;
    }

    Ok(())
  }
}
