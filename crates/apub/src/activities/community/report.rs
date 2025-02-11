use super::report_inboxes;
use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person_in_community},
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{
    activities::community::{
      announce::AnnounceActivity,
      report::{Report, ReportObject},
    },
    InCommunity,
  },
  PostOrComment,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::FlagType,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{check_comment_deleted_or_removed, check_post_deleted_or_removed},
};
use lemmy_db_schema::{
  source::{
    comment_report::{CommentReport, CommentReportForm},
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl Report {
  pub(crate) fn new(
    object_id: &ObjectId<PostOrComment>,
    actor: &ApubPerson,
    community: &ApubCommunity,
    reason: Option<String>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    let kind = FlagType::Flag;
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    Ok(Report {
      actor: actor.id().into(),
      to: [community.id().into()],
      object: ReportObject::Lemmy(object_id.clone()),
      summary: reason,
      content: None,
      kind,
      id: id.clone(),
    })
  }

  pub(crate) async fn send(
    object_id: ObjectId<PostOrComment>,
    actor: &ApubPerson,
    community: &ApubCommunity,
    reason: String,
    context: Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let report = Self::new(&object_id, actor, community, Some(reason), &context)?;
    let inboxes = report_inboxes(object_id, community, &context).await?;

    send_lemmy_activity(&context, report, actor, inboxes, false).await
  }
}

#[async_trait::async_trait]
impl ActivityHandler for Report {
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
    verify_person_in_community(&self.actor, &community, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let actor = self.actor.dereference(context).await?;
    let reason = self.reason()?;
    match self.object.dereference(context).await? {
      PostOrComment::Post(post) => {
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
      PostOrComment::Comment(comment) => {
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
    };

    let community = self.community(context).await?;
    if community.local {
      // forward to remote mods
      let object_id = self.object.object_id(context).await?;
      let announce = AnnouncableActivities::Report(self);
      let announce = AnnounceActivity::new(announce.try_into()?, &community, context)?;
      let inboxes = report_inboxes(object_id, &community, context).await?;
      send_lemmy_activity(context, announce, &community, inboxes.clone(), false).await?;
    }

    Ok(())
  }
}
