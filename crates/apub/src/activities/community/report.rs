use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person_in_community},
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{
    activities::community::report::{Report, ReportObject},
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
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    comment_report::{CommentReport, CommentReportForm},
    community::Community,
    person::Person,
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_utils::error::LemmyError;
use url::Url;

impl Report {
  #[tracing::instrument(skip_all)]
  pub(crate) async fn send(
    object_id: ObjectId<PostOrComment>,
    actor: Person,
    community: Community,
    reason: String,
    context: Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let actor: ApubPerson = actor.into();
    let community: ApubCommunity = community.into();
    let kind = FlagType::Flag;
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let report = Report {
      actor: actor.id().into(),
      to: [community.id().into()],
      object: ReportObject::Lemmy(object_id),
      summary: Some(reason),
      content: None,
      kind,
      id: id.clone(),
      audience: Some(community.id().into()),
    };
    let inbox = if community.local {
      ActivitySendTargets::empty()
    } else {
      ActivitySendTargets::to_inbox(community.shared_inbox_or_inbox())
    };
    send_lemmy_activity(&context, report, &actor, inbox, false).await
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

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    let actor = self.actor.dereference(context).await?;
    let reason = self.reason();
    match self.object.dereference(context).await? {
      PostOrComment::Post(post) => {
        let report_form = PostReportForm {
          creator_id: actor.id,
          post_id: post.id,
          original_post_name: post.name.clone(),
          original_post_url: post.url.clone(),
          reason,
          original_post_body: post.body.clone(),
        };
        PostReport::report(&mut context.pool(), &report_form).await?;
      }
      PostOrComment::Comment(comment) => {
        let report_form = CommentReportForm {
          creator_id: actor.id,
          comment_id: comment.id,
          original_comment_text: comment.content.clone(),
          reason,
        };
        CommentReport::report(&mut context.pool(), &report_form).await?;
      }
    };
    Ok(())
  }
}
