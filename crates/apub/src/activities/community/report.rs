use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person_in_community},
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::community::report::Report,
  ActorType,
  PostOrComment,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor},
};
use activitystreams_kinds::activity::FlagType;
use lemmy_api_common::{comment::CommentReportResponse, post::PostReportResponse, utils::blocking};
use lemmy_db_schema::{
  source::{
    comment_report::{CommentReport, CommentReportForm},
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_db_views::structs::{CommentReportView, PostReportView};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::{messages::SendModRoomMessage, LemmyContext, UserOperation};
use url::Url;

impl Report {
  #[tracing::instrument(skip_all)]
  pub async fn send(
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
      unparsed: Default::default(),
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
    let community = self.to[0]
      .dereference(context, local_instance(context), request_counter)
      .await?;
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
      .dereference(context, local_instance(context), request_counter)
      .await?;
    match self
      .object
      .dereference(context, local_instance(context), request_counter)
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

        let report = blocking(context.pool(), move |conn| {
          PostReport::report(conn, &report_form)
        })
        .await??;

        let post_report_view = blocking(context.pool(), move |conn| {
          PostReportView::read(conn, report.id, actor.id)
        })
        .await??;

        context.chat_server().do_send(SendModRoomMessage {
          op: UserOperation::CreateCommentReport,
          response: PostReportResponse { post_report_view },
          community_id: post.community_id,
          websocket_id: None,
        });
      }
      PostOrComment::Comment(comment) => {
        let report_form = CommentReportForm {
          creator_id: actor.id,
          comment_id: comment.id,
          original_comment_text: comment.content.clone(),
          reason: self.summary,
        };

        let report = blocking(context.pool(), move |conn| {
          CommentReport::report(conn, &report_form)
        })
        .await??;

        let comment_report_view = blocking(context.pool(), move |conn| {
          CommentReportView::read(conn, report.id, actor.id)
        })
        .await??;
        let community_id = comment_report_view.community.id;

        context.chat_server().do_send(SendModRoomMessage {
          op: UserOperation::CreateCommentReport,
          response: CommentReportResponse {
            comment_report_view,
          },
          community_id,
          websocket_id: None,
        });
      }
    };
    Ok(())
  }
}
