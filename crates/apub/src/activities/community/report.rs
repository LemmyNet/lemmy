use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person_in_community},
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::community::report::Report, InCommunity},
  PostOrComment,
  SendActivity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::FlagType,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::{
  comment::{CommentReportResponse, CreateCommentReport},
  context::LemmyContext,
  post::{CreatePostReport, PostReportResponse},
  utils::{local_user_view_from_jwt, sanitize_html},
};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    comment_report::{CommentReport, CommentReportForm},
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait]
impl SendActivity for CreatePostReport {
  type Response = PostReportResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let local_user_view = local_user_view_from_jwt(&request.auth, context).await?;
    Report::send(
      ObjectId::from(response.post_report_view.post.ap_id.clone()),
      &local_user_view.person.into(),
      ObjectId::from(response.post_report_view.community.actor_id.clone()),
      request.reason.to_string(),
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl SendActivity for CreateCommentReport {
  type Response = CommentReportResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let local_user_view = local_user_view_from_jwt(&request.auth, context).await?;
    Report::send(
      ObjectId::from(response.comment_report_view.comment.ap_id.clone()),
      &local_user_view.person.into(),
      ObjectId::from(response.comment_report_view.community.actor_id.clone()),
      request.reason.to_string(),
      context,
    )
    .await
  }
}

impl Report {
  #[tracing::instrument(skip_all)]
  async fn send(
    object_id: ObjectId<PostOrComment>,
    actor: &ApubPerson,
    community_id: ObjectId<ApubCommunity>,
    reason: String,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let community = community_id.dereference_local(context).await?;
    let kind = FlagType::Flag;
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let report = Report {
      actor: actor.id().into(),
      to: [community.id().into()],
      object: object_id,
      summary: reason,
      kind,
      id: id.clone(),
      audience: Some(community.id().into()),
    };
    // todo: this should probably filter and only send if the community is remote?
    let inbox = ActivitySendTargets::to_inbox(community.shared_inbox_or_inbox());
    send_lemmy_activity(context, report, actor, inbox, false).await
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
    match self.object.dereference(context).await? {
      PostOrComment::Post(post) => {
        let report_form = PostReportForm {
          creator_id: actor.id,
          post_id: post.id,
          original_post_name: post.name.clone(),
          original_post_url: post.url.clone(),
          reason: sanitize_html(&self.summary),
          original_post_body: post.body.clone(),
        };
        PostReport::report(&mut context.pool(), &report_form).await?;
      }
      PostOrComment::Comment(comment) => {
        let report_form = CommentReportForm {
          creator_id: actor.id,
          comment_id: comment.id,
          original_comment_text: comment.content.clone(),
          reason: sanitize_html(&self.summary),
        };
        CommentReport::report(&mut context.pool(), &report_form).await?;
      }
    };
    Ok(())
  }
}
