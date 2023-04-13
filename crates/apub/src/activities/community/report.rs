use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person_in_community},
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::community::report::Report, InCommunity},
  ActorType,
  PostOrComment,
  SendActivity,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor},
};
use activitystreams_kinds::activity::FlagType;
use lemmy_api_common::{
  comment::{CommentReportResponse, CreateCommentReport},
  context::LemmyContext,
  post::{CreatePostReport, PostReportResponse},
  utils::get_local_user_view_from_jwt,
  websocket::UserOperation,
};
use lemmy_db_schema::{
  source::{
    comment_report::{CommentReport, CommentReportForm},
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_db_views::structs::{CommentReportView, PostReportView};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait(?Send)]
impl SendActivity for CreatePostReport {
  type Response = PostReportResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&request.auth, context.pool(), context.secret()).await?;
    Report::send(
      ObjectId::new(response.post_report_view.post.ap_id.clone()),
      &local_user_view.person.into(),
      ObjectId::new(response.post_report_view.community.actor_id.clone()),
      request.reason.to_string(),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl SendActivity for CreateCommentReport {
  type Response = CommentReportResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&request.auth, context.pool(), context.secret()).await?;
    Report::send(
      ObjectId::new(response.comment_report_view.comment.ap_id.clone()),
      &local_user_view.person.into(),
      ObjectId::new(response.comment_report_view.community.actor_id.clone()),
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
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community = community_id.dereference_local(context).await?;
    let kind = FlagType::Flag;
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let report = Report {
      actor: ObjectId::new(actor.actor_id()),
      to: [ObjectId::new(community.actor_id())],
      object: object_id,
      summary: reason,
      kind,
      id: id.clone(),
      audience: Some(ObjectId::new(community.actor_id())),
    };

    let inbox = vec![community.shared_inbox_or_inbox()];
    send_lemmy_activity(context, report, actor, inbox, false).await
  }
}

#[async_trait::async_trait(?Send)]
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
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self
      .actor
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    match self
      .object
      .dereference(context, local_instance(context).await, request_counter)
      .await?
    {
      PostOrComment::Post(post) => {
        let report_form = PostReportForm {
          creator_id: actor.id,
          post_id: post.id,
          original_post_name: post.name.clone(),
          original_post_url: post.url.clone(),
          reason: self.summary,
          original_post_body: post.body.clone(),
        };

        let report = PostReport::report(context.pool(), &report_form).await?;

        let post_report_view = PostReportView::read(context.pool(), report.id, actor.id).await?;

        context.send_mod_ws_message(
          &UserOperation::CreateCommentReport,
          &PostReportResponse { post_report_view },
          post.community_id,
          None,
        )?;
      }
      PostOrComment::Comment(comment) => {
        let report_form = CommentReportForm {
          creator_id: actor.id,
          comment_id: comment.id,
          original_comment_text: comment.content.clone(),
          reason: self.summary,
        };

        let report = CommentReport::report(context.pool(), &report_form).await?;

        let comment_report_view =
          CommentReportView::read(context.pool(), report.id, actor.id).await?;
        let community_id = comment_report_view.community.id;

        context.send_mod_ws_message(
          &UserOperation::CreateCommentReport,
          &CommentReportResponse {
            comment_report_view,
          },
          community_id,
          None,
        )?;
      }
    };
    Ok(())
  }
}
